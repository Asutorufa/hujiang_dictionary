use crate::d1::{McpToken, Queries, RawQuery, Word};
use crate::opts::WorkerState;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use d1_orm::DatabaseExecutor;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worker::{Response, Result};

pub const READ_SCOPE: &str = "dictionary:read";
pub const WRITE_SCOPE: &str = "dictionary:write";
const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct TokenMetadata {
    pub id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub last_used_at: Option<i64>,
    pub revoked_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreatedToken {
    #[serde(flatten)]
    pub metadata: TokenMetadata,
    pub token: String,
}

pub async fn create_token(
    opt: &WorkerState,
    request: CreateTokenRequest,
) -> std::result::Result<CreatedToken, String> {
    let name = request.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err("token name must contain 1 to 100 characters".to_string());
    }

    let scopes = normalize_scopes(request.scopes)?;
    let expires_at = match request.expires_in_days {
        Some(days @ 1..=3650) => Some(now() + i64::from(days) * 86_400),
        Some(_) => return Err("expires_in_days must be between 1 and 3650".to_string()),
        None => None,
    };

    let token = generate_token().map_err(|e| e.to_string())?;
    let token_hash = hash_token(&token);
    let id = format!("mcp_{}", &token_hash[..16]);
    let created_at = now();
    let scopes_json = serde_json::to_string(&scopes).map_err(|e| e.to_string())?;

    opt.d1
        .execute(Queries::InsertMcpToken {
            id: &id,
            name: &name,
            token_hash: &token_hash,
            scopes: &scopes_json,
            created_at,
            expires_at,
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(CreatedToken {
        metadata: TokenMetadata {
            id,
            name,
            scopes,
            created_at,
            expires_at,
            last_used_at: None,
            revoked_at: None,
        },
        token,
    })
}

pub async fn list_tokens(opt: &WorkerState) -> std::result::Result<Vec<TokenMetadata>, String> {
    let tokens: Vec<McpToken> = opt
        .d1
        .query_all(Queries::ListMcpTokens)
        .await
        .map_err(|e| e.to_string())?;
    Ok(tokens.iter().map(token_metadata).collect())
}

pub async fn revoke_token(opt: &WorkerState, id: &str) -> std::result::Result<(), String> {
    if id.trim().is_empty() {
        return Err("token id is required".to_string());
    }
    opt.d1
        .execute(Queries::RevokeMcpToken { id })
        .await
        .map_err(|e| e.to_string())
}

fn token_metadata(token: &McpToken) -> TokenMetadata {
    TokenMetadata {
        id: token.id.clone(),
        name: token.name.clone(),
        scopes: parse_scopes(&token.scopes),
        created_at: token.created_at,
        expires_at: token.expires_at,
        last_used_at: token.last_used_at,
        revoked_at: token.revoked_at,
    }
}

fn normalize_scopes(scopes: Vec<String>) -> std::result::Result<Vec<String>, String> {
    let mut normalized = scopes
        .into_iter()
        .map(|scope| scope.trim().to_string())
        .filter(|scope| !scope.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    if normalized.is_empty() {
        return Err("at least one dictionary scope is required".to_string());
    }
    if normalized
        .iter()
        .any(|scope| scope != READ_SCOPE && scope != WRITE_SCOPE)
    {
        return Err("unsupported dictionary scope".to_string());
    }
    Ok(normalized)
}

fn parse_scopes(scopes: &str) -> Vec<String> {
    serde_json::from_str(scopes).unwrap_or_default()
}

fn now() -> i64 {
    Utc::now().timestamp()
}

fn generate_token() -> std::result::Result<String, getrandom::Error> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)?;
    Ok(format!("mcp_{}", URL_SAFE_NO_PAD.encode(bytes)))
}

fn hash_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

fn has_scope(token: &McpToken, required: &str) -> bool {
    parse_scopes(&token.scopes)
        .iter()
        .any(|scope| scope == required)
}

async fn authenticate(
    opt: &WorkerState,
    authorization: Option<&str>,
) -> std::result::Result<McpToken, ()> {
    let token = bearer_token(authorization).ok_or(())?;
    let token_hash = hash_token(token);
    let found: Option<McpToken> = opt
        .d1
        .query_first(Queries::FindMcpToken {
            token_hash: &token_hash,
            now: now(),
        })
        .await
        .map_err(|_| ())?;
    let token = found.ok_or(())?;
    opt.d1
        .execute(Queries::TouchMcpToken {
            id: &token.id,
            now: now(),
        })
        .await
        .map_err(|_| ())?;
    Ok(token)
}

fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    let value = authorization?;
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    let token = token.trim();
    (!token.is_empty()).then_some(token)
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug)]
struct RpcError {
    code: i32,
    message: String,
}

pub async fn handle(
    method: &str,
    origin: Option<&str>,
    authorization: Option<&str>,
    accept: Option<&str>,
    protocol_version: Option<&str>,
    body: Vec<u8>,
    opt: Arc<WorkerState>,
) -> Result<Response> {
    if let Some(origin) = origin
        && !origin.is_empty()
        && !opt
            .mcp_allowed_origins
            .iter()
            .any(|allowed| allowed == origin)
    {
        return json_response(403, json!({"error": "origin is not allowed"}));
    }

    if method == "GET" {
        return method_not_allowed();
    }
    if method != "POST" {
        return json_response(405, json!({"error": "method not allowed"}));
    }

    let accept = accept.unwrap_or_default();
    if !accept.contains("application/json") || !accept.contains("text/event-stream") {
        return json_response(
            406,
            json!({"error": "Accept must include application/json and text/event-stream"}),
        );
    }

    if let Some(version) = protocol_version
        && version != "2025-03-26"
        && version != MCP_PROTOCOL_VERSION
    {
        return json_response(400, json!({"error": "unsupported MCP protocol version"}));
    }

    let token = match authenticate(&opt, authorization).await {
        Ok(token) => token,
        Err(()) => {
            let mut response = json_response(401, json!({"error": "invalid MCP token"}))?;
            response.headers_mut().set("WWW-Authenticate", "Bearer")?;
            return Ok(response);
        }
    };

    let request: JsonRpcRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(_) => return json_response(400, json!({"error": "invalid JSON-RPC request"})),
    };
    if request.jsonrpc != "2.0" {
        return json_response(400, json!({"error": "jsonrpc must be 2.0"}));
    }

    if request.id.is_none() {
        return json_response(202, Value::Null);
    }

    let id = request.id.unwrap_or(Value::Null);
    let response = match dispatch(&opt, &token, &request.method, request.params).await {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": error.code, "message": error.message}
        }),
    };
    json_response(200, response)
}

fn method_not_allowed() -> Result<Response> {
    let mut response = json_response(405, json!({"error": "SSE is not enabled"}))?;
    response.headers_mut().set("Allow", "POST")?;
    Ok(response)
}

async fn dispatch(
    opt: &WorkerState,
    token: &McpToken,
    method: &str,
    params: Value,
) -> std::result::Result<Value, RpcError> {
    match method {
        "initialize" => initialize(params),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tool_definitions()})),
        "tools/call" => call_tool(opt, token, params).await,
        "notifications/initialized" => Ok(Value::Null),
        _ => Err(RpcError {
            code: -32601,
            message: format!("method not found: {method}"),
        }),
    }
}

fn initialize(params: Value) -> std::result::Result<Value, RpcError> {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(MCP_PROTOCOL_VERSION);
    if requested != "2025-03-26" && requested != MCP_PROTOCOL_VERSION {
        return Err(RpcError {
            code: -32602,
            message: "unsupported MCP protocol version".to_string(),
        });
    }
    Ok(json!({
        "protocolVersion": requested,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {"name": "hj-dictionary", "version": env!("CARGO_PKG_VERSION")}
    }))
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "dictionary.search",
            "description": "Search saved dictionary entries with pagination.",
            "inputSchema": {"type": "object", "properties": {
                "query": {"type": "string"},
                "type": {"type": "integer"},
                "page_size": {"type": "integer", "minimum": 1, "maximum": 100},
                "page_number": {"type": "integer", "minimum": 1},
                "order_by": {"type": "string", "enum": ["word", "update_time", "priority", "reminder_time", "anki_count", "add_time"]},
                "descending": {"type": "boolean"}
            }}
        }),
        json!({
            "name": "dictionary.get",
            "description": "Get one saved dictionary entry by exact word.",
            "inputSchema": {"type": "object", "required": ["word"], "properties": {"word": {"type": "string"}}}
        }),
        json!({
            "name": "dictionary.save",
            "description": "Create or update one dictionary entry.",
            "inputSchema": entry_schema()
        }),
        json!({
            "name": "dictionary.rename",
            "description": "Rename an existing dictionary entry and update its content.",
            "inputSchema": {"type": "object", "required": ["old_word", "word", "explain"], "properties": {
                "old_word": {"type": "string"}, "word": {"type": "string"}, "explain": {"type": "string"},
                "example": {"type": "string"}, "type": {"type": "integer"}
            }}
        }),
        json!({
            "name": "dictionary.delete",
            "description": "Delete one dictionary entry.",
            "inputSchema": {"type": "object", "required": ["word"], "properties": {"word": {"type": "string"}}}
        }),
        json!({
            "name": "dictionary.set_priority",
            "description": "Set the priority of one dictionary entry.",
            "inputSchema": {"type": "object", "required": ["word", "priority"], "properties": {
                "word": {"type": "string"}, "priority": {"type": "integer", "minimum": 0}
            }}
        }),
        json!({
            "name": "dictionary.preview_changes",
            "description": "Validate and preview a batch of dictionary changes without writing them.",
            "inputSchema": change_set_schema()
        }),
        json!({
            "name": "dictionary.apply_changes",
            "description": "Apply a validated batch of dictionary changes.",
            "inputSchema": change_set_schema()
        }),
    ]
}

fn entry_schema() -> Value {
    json!({"type": "object", "required": ["word", "explain"], "properties": {
        "word": {"type": "string"}, "explain": {"type": "string"}, "example": {"type": "string"}, "type": {"type": "integer"}, "priority": {"type": "integer", "minimum": 0}
    }})
}

fn change_set_schema() -> Value {
    json!({"type": "object", "required": ["changes"], "properties": {"changes": {"type": "array", "maxItems": 100, "items": {"type": "object", "required": ["operation"], "properties": {
        "operation": {"type": "string", "enum": ["save", "rename", "delete", "set_priority"]},
        "word": {"type": "string"}, "old_word": {"type": "string"}, "explain": {"type": "string"}, "example": {"type": "string"}, "type": {"type": "integer"}, "priority": {"type": "integer", "minimum": 0}
    }}}}})
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: Option<String>,
    #[serde(rename = "type")]
    word_type: Option<i64>,
    page_size: Option<u64>,
    page_number: Option<u64>,
    order_by: Option<String>,
    descending: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct WordArgs {
    word: String,
}

#[derive(Debug, Deserialize)]
struct SaveArgs {
    word: String,
    explain: String,
    example: Option<String>,
    #[serde(rename = "type")]
    word_type: Option<i64>,
    priority: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RenameArgs {
    old_word: String,
    word: String,
    explain: String,
    example: Option<String>,
    #[serde(rename = "type")]
    word_type: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PriorityArgs {
    word: String,
    priority: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
enum DictionaryChange {
    Save {
        word: String,
        explain: String,
        example: Option<String>,
        #[serde(rename = "type")]
        word_type: Option<i64>,
        priority: Option<u64>,
    },
    Rename {
        old_word: String,
        word: String,
        explain: String,
        example: Option<String>,
        #[serde(rename = "type")]
        word_type: Option<i64>,
    },
    Delete {
        word: String,
    },
    SetPriority {
        word: String,
        priority: u64,
    },
}

#[derive(Debug, Deserialize)]
struct ChangeSetArgs {
    changes: Vec<DictionaryChange>,
}

async fn call_tool(
    opt: &WorkerState,
    token: &McpToken,
    params: Value,
) -> std::result::Result<Value, RpcError> {
    let name = params.get("name").and_then(Value::as_str).ok_or(RpcError {
        code: -32602,
        message: "tools/call requires a tool name".to_string(),
    })?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = match name {
        "dictionary.search" => {
            require_scope(token, READ_SCOPE)?;
            let args: SearchArgs = parse_args(arguments)?;
            let query = search_word_query(
                args.query.as_deref().unwrap_or_default(),
                args.page_size.unwrap_or(20),
                args.page_number.unwrap_or(1),
                args.order_by.as_deref().unwrap_or("word"),
                args.descending.unwrap_or(false),
                args.word_type.unwrap_or(0),
            )
            .map_err(tool_error)?;
            let words: Vec<Word> = opt.d1.query_all(query).await.map_err(tool_error)?;
            tool_text(serde_json::to_string(&words).map_err(tool_error)?)
        }
        "dictionary.get" => {
            require_scope(token, READ_SCOPE)?;
            let args: WordArgs = parse_args(arguments)?;
            let word: Option<Word> = opt
                .d1
                .query_first(Queries::GetWord { word: &args.word })
                .await
                .map_err(tool_error)?;
            tool_text(serde_json::to_string(&word).map_err(tool_error)?)
        }
        "dictionary.save" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: SaveArgs = parse_args(arguments)?;
            save_word(opt, &args).await.map_err(tool_error)?;
            tool_text("dictionary entry saved".to_string())
        }
        "dictionary.rename" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: RenameArgs = parse_args(arguments)?;
            rename_word(opt, &args).await.map_err(tool_error)?;
            tool_text("dictionary entry renamed".to_string())
        }
        "dictionary.delete" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: WordArgs = parse_args(arguments)?;
            opt.d1
                .execute(Queries::DeleteWord { word: &args.word })
                .await
                .map_err(tool_error)?;
            tool_text("dictionary entry deleted".to_string())
        }
        "dictionary.set_priority" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: PriorityArgs = parse_args(arguments)?;
            opt.d1
                .execute(Queries::ChangePriority {
                    priority: args.priority,
                    word: &args.word,
                })
                .await
                .map_err(tool_error)?;
            tool_text("dictionary priority updated".to_string())
        }
        "dictionary.preview_changes" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: ChangeSetArgs = parse_args(arguments)?;
            validate_changes(&args.changes).map_err(tool_error)?;
            tool_text(
                serde_json::to_string(&json!({
                    "valid": true,
                    "change_count": args.changes.len(),
                    "changes": args.changes,
                }))
                .map_err(tool_error)?,
            )
        }
        "dictionary.apply_changes" => {
            require_scope(token, WRITE_SCOPE)?;
            let args: ChangeSetArgs = parse_args(arguments)?;
            validate_changes(&args.changes).map_err(tool_error)?;
            for change in &args.changes {
                apply_change(opt, change).await.map_err(tool_error)?;
            }
            tool_text(format!("applied {} dictionary changes", args.changes.len()))
        }
        _ => {
            return Err(RpcError {
                code: -32602,
                message: format!("unknown tool: {name}"),
            });
        }
    }?;

    Ok(json!({"content": [{"type": "text", "text": result}], "isError": false}))
}

fn require_scope(token: &McpToken, scope: &str) -> std::result::Result<(), RpcError> {
    if has_scope(token, scope) {
        Ok(())
    } else {
        Err(RpcError {
            code: -32003,
            message: format!("missing scope: {scope}"),
        })
    }
}

fn parse_args<T: for<'de> Deserialize<'de>>(value: Value) -> std::result::Result<T, RpcError> {
    serde_json::from_value(value).map_err(|e| RpcError {
        code: -32602,
        message: e.to_string(),
    })
}

fn tool_error(error: impl ToString) -> RpcError {
    RpcError {
        code: -32000,
        message: error.to_string(),
    }
}

fn tool_text(text: String) -> std::result::Result<String, RpcError> {
    Ok(text)
}

async fn save_word(opt: &WorkerState, args: &SaveArgs) -> std::result::Result<(), String> {
    if args.word.trim().is_empty() || args.explain.trim().is_empty() {
        return Err("word and explain are required".to_string());
    }
    opt.d1
        .execute(Queries::SaveWord {
            word: &args.word,
            explain: &args.explain,
            word_type: args.word_type.unwrap_or(0),
            example: args.example.as_deref().unwrap_or_default(),
        })
        .await
        .map_err(|e| e.to_string())?;
    if let Some(priority) = args.priority {
        opt.d1
            .execute(Queries::ChangePriority {
                priority,
                word: &args.word,
            })
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn rename_word(opt: &WorkerState, args: &RenameArgs) -> std::result::Result<(), String> {
    if args.old_word.trim().is_empty()
        || args.word.trim().is_empty()
        || args.explain.trim().is_empty()
    {
        return Err("old_word, word, and explain are required".to_string());
    }
    opt.d1
        .execute(Queries::RenameWord {
            new_word: &args.word,
            explain: &args.explain,
            word_type: args.word_type.unwrap_or(0),
            example: args.example.as_deref().unwrap_or_default(),
            old_word: &args.old_word,
        })
        .await
        .map_err(|e| e.to_string())
}

fn validate_changes(changes: &[DictionaryChange]) -> std::result::Result<(), String> {
    if changes.is_empty() || changes.len() > 100 {
        return Err("changes must contain 1 to 100 operations".to_string());
    }
    for change in changes {
        match change {
            DictionaryChange::Save { word, explain, .. } => {
                if word.trim().is_empty() || explain.trim().is_empty() {
                    return Err("save requires word and explain".to_string());
                }
            }
            DictionaryChange::Rename {
                old_word,
                word,
                explain,
                ..
            } => {
                if old_word.trim().is_empty() || word.trim().is_empty() || explain.trim().is_empty()
                {
                    return Err("rename requires old_word, word, and explain".to_string());
                }
            }
            DictionaryChange::Delete { word } | DictionaryChange::SetPriority { word, .. } => {
                if word.trim().is_empty() {
                    return Err("word is required".to_string());
                }
            }
        }
    }
    Ok(())
}

async fn apply_change(
    opt: &WorkerState,
    change: &DictionaryChange,
) -> std::result::Result<(), String> {
    match change {
        DictionaryChange::Save {
            word,
            explain,
            example,
            word_type,
            priority,
        } => {
            save_word(
                opt,
                &SaveArgs {
                    word: word.clone(),
                    explain: explain.clone(),
                    example: example.clone(),
                    word_type: *word_type,
                    priority: *priority,
                },
            )
            .await
        }
        DictionaryChange::Rename {
            old_word,
            word,
            explain,
            example,
            word_type,
        } => {
            rename_word(
                opt,
                &RenameArgs {
                    old_word: old_word.clone(),
                    word: word.clone(),
                    explain: explain.clone(),
                    example: example.clone(),
                    word_type: *word_type,
                },
            )
            .await
        }
        DictionaryChange::Delete { word } => opt
            .d1
            .execute(Queries::DeleteWord { word })
            .await
            .map_err(|e| e.to_string()),
        DictionaryChange::SetPriority { word, priority } => opt
            .d1
            .execute(Queries::ChangePriority {
                priority: *priority,
                word,
            })
            .await
            .map_err(|e| e.to_string()),
    }
}

fn json_response(status: u16, body: Value) -> Result<Response> {
    let mut response = Response::from_bytes(serde_json::to_vec(&body)?)?.with_status(status);
    response
        .headers_mut()
        .set("Content-Type", "application/json")?;
    Ok(response)
}

fn search_word_query(
    query: &str,
    page_size: u64,
    page_number: u64,
    order_by: &str,
    is_desc: bool,
    word_type: i64,
) -> std::result::Result<RawQuery, String> {
    let limit = page_size.clamp(1, 100);
    let offset = (page_number.max(1) - 1) * limit;
    let safe_order_by = match order_by {
        "word" | "update_time" | "priority" | "reminder_time" | "anki_count" | "add_time" => {
            order_by
        }
        _ => return Err("invalid order_by".to_string()),
    };
    let direction = if is_desc { " DESC" } else { "" };
    let query = query.trim();
    let sql = if query.is_empty() {
        format!(
            "SELECT * FROM words WHERE word_type = ? ORDER BY {}{} LIMIT ? OFFSET ?",
            safe_order_by, direction
        )
    } else {
        format!(
            "SELECT * FROM words WHERE word_type = ? AND (word LIKE ? OR explain LIKE ? OR example LIKE ?) ORDER BY {}{} LIMIT ? OFFSET ?",
            safe_order_by, direction
        )
    };
    let pattern = format!("%{query}%");
    let mut params = vec![word_type.into()];
    if !query.is_empty() {
        params.extend([
            pattern.clone().into(),
            pattern.clone().into(),
            pattern.into(),
        ]);
    }
    params.extend([((limit) as i64).into(), (offset as i64).into()]);
    Ok(RawQuery { sql, params })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_are_normalized_and_deduplicated() {
        assert_eq!(
            normalize_scopes(vec![
                WRITE_SCOPE.to_string(),
                READ_SCOPE.to_string(),
                WRITE_SCOPE.to_string(),
            ])
            .unwrap(),
            vec![READ_SCOPE, WRITE_SCOPE]
        );
        assert!(normalize_scopes(vec![]).is_err());
        assert!(normalize_scopes(vec!["admin".to_string()]).is_err());
    }

    #[test]
    fn change_validation_rejects_empty_or_oversized_batches() {
        assert!(validate_changes(&[]).is_err());
        assert!(
            validate_changes(&[DictionaryChange::Delete {
                word: " ".to_string(),
            }])
            .is_err()
        );

        let changes = (0..=100)
            .map(|_| DictionaryChange::Delete {
                word: "word".to_string(),
            })
            .collect::<Vec<_>>();
        assert!(validate_changes(&changes).is_err());
    }

    #[test]
    fn search_query_whitelists_ordering_and_clamps_page_size() {
        let query = search_word_query("hello", 500, 0, "priority", true, 0).unwrap();
        assert!(query.sql.contains("ORDER BY priority DESC"));
        assert_eq!(query.params.len(), 6);
        assert!(search_word_query("", 10, 1, "word; DROP TABLE words", false, 0).is_err());
    }

    #[test]
    fn token_hash_is_stable_but_changes_with_input() {
        assert_eq!(hash_token("token"), hash_token("token"));
        assert_ne!(hash_token("token"), hash_token("other-token"));
    }

    #[test]
    fn bearer_scheme_is_case_insensitive() {
        assert_eq!(bearer_token(Some("Bearer token")), Some("token"));
        assert_eq!(bearer_token(Some("bearer token")), Some("token"));
        assert_eq!(bearer_token(Some("Basic token")), None);
        assert_eq!(bearer_token(Some("Bearer")), None);
    }
}
