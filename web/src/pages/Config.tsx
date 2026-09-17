import { useEffect, useState } from "react";
import {
  Button,
  Card,
  TextField,
  Dialog,
  Flex,
  Text,
  Box,
  Spinner,
  Switch,
} from "@radix-ui/themes";
import { addToast, ConfirmModal, useDisclosure } from "@/components";
import { authorizedRequest } from "../lib/api";
import { useLocation } from "wouter";
import { ROUTE_LOGIN } from "../lib/constants";
import { EmptyState } from "@/ui/EmptyState";
import { PageContainer } from "@/ui/PageContainer";
import {
  ArrowUpRight,
  Check,
  Copy,
  ExternalLink,
  KeyRound,
  PlugZap,
  Search,
  Send,
  ShieldCheck,
} from "lucide-react";

export type LlmProvider = {
  name: string;
  base_url: string;
  api_key: string;
  provider: string;
  models: string;
  project_id: string;
  location: string;
  features: string;
};

export type Configuration = {
  key: string;
  value: string;
};

type McpToken = {
  id: string;
  name: string;
  scopes: string[];
  created_at: number;
  expires_at: number | null;
  last_used_at: number | null;
  revoked_at: number | null;
};

type CreatedMcpToken = McpToken & {
  token: string;
};

const normalizeProviderType = (provider: string) =>
  provider === "claude"
    ? "anthropic"
    : provider === "openai-codex"
      ? "codex"
      : provider;

const providerLabel = (provider: string) =>
  normalizeProviderType(provider) === "codex"
    ? "OpenAI Codex (ChatGPT)"
    : normalizeProviderType(provider);

const isCodexConnected = (provider: LlmProvider | null) => {
  if (!provider || normalizeProviderType(provider.provider) !== "codex") {
    return false;
  }
  try {
    const features = JSON.parse(provider.features || "{}") as {
      oauth?: { connected?: boolean };
    };
    return features.oauth?.connected === true;
  } catch {
    return false;
  }
};

type CodexLoginState = {
  state: string;
  user_code: string;
  verification_url: string;
  interval_seconds: number;
};

type CodexPollResponse = {
  status: "pending" | "connected";
  retry_after_seconds?: number;
};

export default function Config() {
  const [, setLocation] = useLocation();
  const [providers, setProviders] = useState<LlmProvider[]>([]);
  const [providersLoading, setProvidersLoading] = useState(false);
  const [configurations, setConfigurations] = useState<Configuration[]>([]);
  const [mcpTokens, setMcpTokens] = useState<McpToken[]>([]);
  const [activeTab, setActiveTab] = useState("llm");
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [revokeTokenId, setRevokeTokenId] = useState<string | null>(null);
  const [tokenDialogOpen, setTokenDialogOpen] = useState(false);
  const [createdToken, setCreatedToken] = useState<CreatedMcpToken | null>(
    null,
  );
  const [tokenName, setTokenName] = useState("");
  const [tokenExpiry, setTokenExpiry] = useState("90");
  const [tokenScopes, setTokenScopes] = useState({
    read: true,
    write: true,
  });

  const { isOpen, onOpenChange } = useDisclosure();
  const [editingProvider, setEditingProvider] = useState<LlmProvider | null>(
    null,
  );
  const [selectedProviderType, setSelectedProviderType] =
    useState<string>("openai");
  const [geminiSearch, setGeminiSearch] = useState<boolean>(false);
  const [projectId, setProjectId] = useState<string>("");
  const [vertexLocation, setVertexLocation] = useState<string>("");
  const [anthropicVersion, setAnthropicVersion] =
    useState<string>("2023-06-01");
  const [thinkingEnabled, setThinkingEnabled] = useState<boolean>(false);
  const [thinkingBudget, setThinkingBudget] = useState<string>("1024");
  const [codexLogin, setCodexLogin] = useState<CodexLoginState | null>(null);
  const [codexLoginLoading, setCodexLoginLoading] = useState(false);
  const [codexCodeCopied, setCodexCodeCopied] = useState(false);

  const fetchProviders = async () => {
    setProvidersLoading(true);
    try {
      const res = await authorizedRequest("/llm/list", {
        method: "POST",
      });
      if (!res.ok) {
        if (res.status === 401) {
          setLocation(ROUTE_LOGIN);
        }
        return;
      }
      const data = (await res.json()) as LlmProvider[];
      setProviders(data);
    } catch (e) {
      console.error(e);
    } finally {
      setProvidersLoading(false);
    }
  };

  const fetchConfigurations = async () => {
    try {
      const res = await authorizedRequest("/config/list", {
        method: "POST",
      });
      if (!res.ok) {
        if (res.status === 401) {
          setLocation(ROUTE_LOGIN);
        }
        return;
      }
      const data = (await res.json()) as Configuration[];
      setConfigurations(data);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchMcpTokens = async () => {
    try {
      const res = await authorizedRequest("/mcp/tokens/list", {
        method: "POST",
      });
      if (!res.ok) {
        if (res.status === 401) setLocation(ROUTE_LOGIN);
        return;
      }
      setMcpTokens((await res.json()) as McpToken[]);
    } catch (e) {
      console.error(e);
    }
  };

  const ensureOk = async (res: Response, action: string) => {
    if (!res.ok) {
      throw new Error(`${action} failed (${res.status}): ${await res.text()}`);
    }
  };

  useEffect(() => {
    fetchProviders();
    fetchConfigurations();
    fetchMcpTokens();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!codexLogin) return;

    let cancelled = false;
    const wait = (milliseconds: number) =>
      new Promise<void>((resolve) => {
        window.setTimeout(resolve, milliseconds);
      });

    const poll = async () => {
      try {
        let retryAfter = Math.max(1, codexLogin.interval_seconds);
        while (!cancelled) {
          await wait(retryAfter * 1000);
          if (cancelled) return;

          const res = await authorizedRequest("/llm/codex/poll", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ state: codexLogin.state }),
          });
          await ensureOk(res, "Check Codex sign-in");
          const result = (await res.json()) as CodexPollResponse;

          if (result.status === "connected") {
            setCodexLogin(null);
            setCodexCodeCopied(false);
            addToast({
              title: "OpenAI Codex connected",
              description: "Your ChatGPT credentials are stored securely.",
              color: "success",
            });
            onOpenChange(false);
            await fetchProviders();
            return;
          }

          retryAfter = Math.max(
            1,
            result.retry_after_seconds || codexLogin.interval_seconds,
          );
        }
      } catch (err) {
        if (!cancelled) {
          setCodexLogin(null);
          setCodexCodeCopied(false);
          addToast({
            title: "Codex sign-in failed",
            description: err instanceof Error ? err.message : String(err),
            color: "danger",
          });
        }
      }
    };

    void poll();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [codexLogin?.state]);

  const handleCreateMcpToken = async (
    event: React.FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    const scopes = [
      tokenScopes.read ? "dictionary:read" : "",
      tokenScopes.write ? "dictionary:write" : "",
    ].filter(Boolean);
    if (scopes.length === 0) {
      addToast({
        title: "Select at least one permission",
        color: "warning",
      });
      return;
    }

    try {
      const res = await authorizedRequest("/mcp/tokens/create", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: tokenName.trim(),
          scopes,
          expires_in_days: tokenExpiry === "never" ? null : Number(tokenExpiry),
        }),
      });
      await ensureOk(res, "Create MCP token");
      const token = (await res.json()) as CreatedMcpToken;
      setTokenDialogOpen(false);
      setCreatedToken(token);
      setTokenName("");
      await fetchMcpTokens();
    } catch (err) {
      addToast({
        title: "Failed to create MCP token",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
      });
    }
  };

  const handleRevokeMcpToken = async () => {
    if (!revokeTokenId) return;
    try {
      const res = await authorizedRequest("/mcp/tokens/revoke", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: revokeTokenId }),
      });
      await ensureOk(res, "Revoke MCP token");
      addToast({ title: "MCP token revoked", color: "success" });
      await fetchMcpTokens();
    } catch (err) {
      addToast({
        title: "Failed to revoke MCP token",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
      });
    } finally {
      setRevokeTokenId(null);
    }
  };

  const formatTokenDate = (timestamp: number | null) =>
    timestamp ? new Date(timestamp * 1000).toLocaleString() : "Never";

  const handleCopyCodexCode = async () => {
    if (!codexLogin) return;
    try {
      if (!navigator.clipboard) {
        throw new Error("Clipboard access is unavailable");
      }
      await navigator.clipboard.writeText(codexLogin.user_code);
      setCodexCodeCopied(true);
      window.setTimeout(() => setCodexCodeCopied(false), 1600);
    } catch {
      addToast({
        title: "Could not copy the device code",
        description: "Copy the highlighted code manually.",
        color: "warning",
      });
    }
  };

  const handleDelete = (name: string) => {
    setDeleteTarget(name);
  };

  const handleSave = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const data = Object.fromEntries(
      formData.entries(),
    ) as unknown as LlmProvider;

    if (data.provider === "codex" && !isCodexConnected(editingProvider)) {
      try {
        setCodexLoginLoading(true);
        const res = await authorizedRequest("/llm/codex/login", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: data.name.trim(),
            models: data.models.trim(),
          }),
        });
        await ensureOk(res, "Start Codex sign-in");
        const login = (await res.json()) as CodexLoginState;
        if (!login.state || !login.user_code || !login.verification_url) {
          throw new Error("Codex sign-in returned an invalid device code");
        }
        setCodexLogin(login);
        setCodexCodeCopied(false);
        addToast({
          title: "Finish signing in to OpenAI",
          description: "Use the device code shown in this dialog.",
          color: "info",
        });
      } catch (err) {
        console.error(err);
        addToast({
          title: "Failed to start Codex sign-in",
          description: err instanceof Error ? err.message : String(err),
          color: "danger",
        });
      } finally {
        setCodexLoginLoading(false);
      }
      return;
    }

    // Pack features
    let featuresObj: Record<string, unknown> = {};
    if (editingProvider && editingProvider.features) {
      try {
        featuresObj = JSON.parse(editingProvider.features) as Record<
          string,
          unknown
        >;
      } catch {
        addToast({
          title: "Warning",
          description:
            "Existing feature configuration was corrupt and will be reset.",
          color: "warning",
        });
      }
    }

    if (
      (data.provider === "gemini" || data.provider === "vertexai") &&
      geminiSearch
    ) {
      featuresObj.gemini_search = true;
    } else {
      delete featuresObj.gemini_search;
    }

    if (data.provider === "vertexai") {
      const formDataProjectId = formData.get("project_id") as string;
      const formDataLocation = formData.get("location") as string;
      if (formDataProjectId) featuresObj.project_id = formDataProjectId;
      if (formDataLocation) featuresObj.location = formDataLocation;
    } else {
      delete featuresObj.project_id;
      delete featuresObj.location;
    }

    if (data.provider === "anthropic") {
      const formDataVersion = formData.get("anthropic_version") as string;
      if (formDataVersion) featuresObj.anthropic_version = formDataVersion;
      if (thinkingEnabled) {
        featuresObj.thinking = {
          budget_tokens: Math.max(1024, parseInt(thinkingBudget, 10) || 1024),
        };
      } else {
        delete featuresObj.thinking;
      }
    } else {
      delete featuresObj.anthropic_version;
      delete featuresObj.thinking;
    }

    data.features = JSON.stringify(featuresObj);

    try {
      const res = await authorizedRequest("/llm/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      await ensureOk(res, "Save provider");
      onOpenChange(false);
      await fetchProviders();
    } catch (err) {
      console.error(err);
      addToast({
        title: "Failed to save provider",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
      });
    }
  };

  const openAddModal = () => {
    setEditingProvider(null);
    setCodexLogin(null);
    setCodexCodeCopied(false);
    setSelectedProviderType("openai");
    setGeminiSearch(false);
    setProjectId("");
    setVertexLocation("");
    setAnthropicVersion("2023-06-01");
    setThinkingEnabled(false);
    setThinkingBudget("1024");
    onOpenChange(true);
  };

  const handleGeneralConfigSave = async (
    e: React.FormEvent<HTMLFormElement>,
  ) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);

    const telegramToken = formData.get("TELEGRAM_TOKEN") as string;
    const allowUsers = formData.get("ALLOW_USERS") as string;
    const maintainerId = formData.get("MAINTAINER_ID") as string;
    const googleSearchApiKey = formData.get("GOOGLE_SEARCH_API_KEY") as string;
    const googleSearchCx = formData.get("GOOGLE_SEARCH_CX") as string;

    const formattedAllowUsers = allowUsers
      .split(",")
      .map((u) => u.trim())
      .filter((u) => u.length > 0)
      .join(",");

    const updates = [
      { key: "TELEGRAM_TOKEN", value: telegramToken.trim() },
      { key: "ALLOW_USERS", value: formattedAllowUsers },
      { key: "MAINTAINER_ID", value: maintainerId.trim() },
      { key: "GOOGLE_SEARCH_API_KEY", value: googleSearchApiKey.trim() },
      { key: "GOOGLE_SEARCH_CX", value: googleSearchCx.trim() },
    ];

    try {
      await Promise.all(
        updates.map(async (config) => {
          const res = await authorizedRequest("/config/save", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(config),
          });
          await ensureOk(res, `Save ${config.key}`);
        }),
      );
      await fetchConfigurations();
      addToast({
        title: "Configurations saved",
        color: "success",
      });
    } catch (err) {
      console.error(err);
      addToast({
        title: "Failed to save configurations",
        description: err instanceof Error ? err.message : String(err),
        color: "danger",
        timeout: 0,
      });
    }
  };

  const openEditModal = (provider: LlmProvider) => {
    setEditingProvider(provider);
    setCodexLogin(null);
    setCodexCodeCopied(false);
    setSelectedProviderType(normalizeProviderType(provider.provider));
    try {
      const features = JSON.parse(provider.features || "{}") as Record<
        string,
        unknown
      >;
      setGeminiSearch(features?.gemini_search === true);
      setProjectId((features?.project_id as string) || "");
      setVertexLocation((features?.location as string) || "");
      setAnthropicVersion(
        (features?.anthropic_version as string) || "2023-06-01",
      );
      setThinkingEnabled(!!features?.thinking);
      if (features?.thinking && typeof features.thinking === "object") {
        setThinkingBudget(
          String(
            (features.thinking as { budget_tokens?: number }).budget_tokens ||
              "1024",
          ),
        );
      } else {
        setThinkingBudget("1024");
      }
    } catch {
      setGeminiSearch(false);
      setProjectId("");
      setVertexLocation("");
      setAnthropicVersion("2023-06-01");
      setThinkingEnabled(false);
      setThinkingBudget("1024");
    }
    onOpenChange(true);
  };

  return (
    <PageContainer size="4xl" className="app-settings-page space-y-6">
      <ConfirmModal
        title={
          deleteTarget
            ? `Are you sure you want to delete provider "${deleteTarget}"?`
            : "Are you sure?"
        }
        open={!!deleteTarget}
        color="danger"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
        onConfirm={async () => {
          if (!deleteTarget) return;
          try {
            const res = await authorizedRequest("/llm/delete", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ name: deleteTarget }),
            });
            if (!res.ok) {
              addToast({
                title: "Delete failed",
                description: `(${res.status}) ${await res.text()}`,
                color: "danger",
              });
              return;
            }
            addToast({
              title: "Provider deleted",
              color: "success",
            });
            fetchProviders();
          } catch (e) {
            addToast({
              title: "Delete failed",
              description: e instanceof Error ? e.message : String(e),
              color: "danger",
            });
          } finally {
            setDeleteTarget(null);
          }
        }}
      />

      <ConfirmModal
        title="Revoke this MCP token?"
        open={!!revokeTokenId}
        color="danger"
        confirmLabel="Revoke"
        cancelLabel="Cancel"
        onChange={(open) => {
          if (!open) setRevokeTokenId(null);
        }}
        onConfirm={handleRevokeMcpToken}
      />

      <header className="app-settings-hero">
        <div className="app-library-eyebrow">Workspace / Settings</div>
        <div className="app-settings-hero-row">
          <div>
            <h1>Settings</h1>
            <p>Shape how DictDeck connects to models and your workspace.</p>
          </div>
          <span className="app-settings-hero-status">Private workspace</span>
        </div>
      </header>

      <div className="app-settings-layout">
        <nav className="app-settings-nav" aria-label="Settings sections">
          <div className="app-settings-nav-label">Manage</div>
          <button
            type="button"
            className={activeTab === "llm" ? "is-active" : ""}
            onClick={() => setActiveTab("llm")}
          >
            <span>LLM providers</span>
            <small>Models and API connections</small>
          </button>
          <button
            type="button"
            className={activeTab === "config" ? "is-active" : ""}
            onClick={() => setActiveTab("config")}
          >
            <span>General</span>
            <small>Workspace-wide settings</small>
          </button>
        </nav>

        <section className="app-settings-content">
          {activeTab === "llm" && (
            <>
              <div className="app-settings-section-header">
                <div>
                  <div className="app-settings-section-kicker">Connections</div>
                  <h2>LLM providers</h2>
                  <p>Choose which model services are available to DictDeck.</p>
                </div>
                <Button
                  color="gray"
                  className="app-primary-action"
                  onClick={openAddModal}
                >
                  Add provider
                </Button>
              </div>

              {providersLoading ? (
                <div className="app-settings-inline-state" role="status">
                  <Spinner size="2" />
                  <span>Loading providers…</span>
                </div>
              ) : providers.length > 0 ? (
                <div className="app-provider-grid">
                  {providers.map((p) => (
                    <Card key={p.name} size="2" className="app-provider-card">
                      <Flex justify="between" align="start" gap="4">
                        <Box className="app-provider-details">
                          <div className="app-provider-kicker">
                            {normalizeProviderType(p.provider) === "codex"
                              ? "ChatGPT OAuth"
                              : "Connected provider"}
                          </div>
                          <Text size="4" weight="bold" as="div">
                            {p.name}
                          </Text>
                          <Text size="2" color="gray" as="div">
                            {providerLabel(p.provider)}
                          </Text>
                          {p.base_url && (
                            <Text size="1" className="truncate" as="div">
                              {p.base_url}
                            </Text>
                          )}
                        </Box>
                        <Flex gap="2" className="shrink-0">
                          <Button
                            size="1"
                            variant="soft"
                            onClick={() => openEditModal(p)}
                          >
                            Edit
                          </Button>
                          <Button
                            size="1"
                            color="red"
                            variant="soft"
                            onClick={() => handleDelete(p.name)}
                          >
                            Delete
                          </Button>
                        </Flex>
                      </Flex>
                    </Card>
                  ))}
                </div>
              ) : (
                <EmptyState
                  title="No providers connected"
                  description="Add a model provider to start using assisted lookups."
                  icon={<PlugZap size={24} />}
                  actionLabel="Add provider"
                  onAction={openAddModal}
                />
              )}
            </>
          )}

          {activeTab === "config" && (
            <div>
              <div className="app-settings-section-header">
                <div>
                  <div className="app-settings-section-kicker">Workspace</div>
                  <h2>General settings</h2>
                  <p>
                    Keep the shared integrations used by your workspace in one
                    place.
                  </p>
                </div>
              </div>
              <div className="app-settings-form">
                <form onSubmit={handleGeneralConfigSave}>
                  <div className="app-settings-panel-stack">
                    <section className="app-settings-panel">
                      <div className="app-settings-panel-heading">
                        <span className="app-settings-panel-icon">
                          <Send size={17} />
                        </span>
                        <div>
                          <h3>Telegram delivery</h3>
                          <p>Control notifications and who can use the bot.</p>
                        </div>
                      </div>
                      <div className="app-settings-fields">
                        <Flex direction="column" gap="1">
                          <Text as="label" size="2" weight="bold">
                            Bot token
                          </Text>
                          <TextField.Root
                            name="TELEGRAM_TOKEN"
                            type="password"
                            defaultValue={
                              configurations.find((c) => c.key === "TELEGRAM_TOKEN")
                                ?.value || ""
                            }
                            placeholder="123456789:ABCDefghIJKlmnopQRSTuvwxYZ1234567890"
                          />
                          <Text size="1" color="gray">
                            Stored securely for Telegram notifications.
                          </Text>
                        </Flex>
                        <Flex direction="column" gap="1">
                          <Text as="label" size="2" weight="bold">
                            Maintainer ID
                          </Text>
                          <TextField.Root
                            name="MAINTAINER_ID"
                            defaultValue={
                              configurations.find((c) => c.key === "MAINTAINER_ID")
                                ?.value || ""
                            }
                            placeholder="123456789"
                          />
                          <Text size="1" color="gray">
                            The primary admin who receives cron messages and error reports.
                          </Text>
                        </Flex>
                        <Flex direction="column" gap="1" className="app-settings-field-wide">
                          <Text as="label" size="2" weight="bold">
                            Allowed users
                          </Text>
                          <TextField.Root
                            name="ALLOW_USERS"
                            defaultValue={
                              configurations.find((c) => c.key === "ALLOW_USERS")
                                ?.value || ""
                            }
                            placeholder="123456789,987654321"
                          />
                          <Text size="1" color="gray">
                            Comma-separated Telegram user IDs.
                          </Text>
                        </Flex>
                      </div>
                    </section>

                    <section className="app-settings-panel">
                      <div className="app-settings-panel-heading">
                        <span className="app-settings-panel-icon">
                          <Search size={17} />
                        </span>
                        <div>
                          <h3>Web search</h3>
                          <p>Allow grounded lookups to use Google Custom Search.</p>
                        </div>
                      </div>
                      <div className="app-settings-fields">
                        <Flex direction="column" gap="1">
                          <Text as="label" size="2" weight="bold">
                            API key
                          </Text>
                          <TextField.Root
                            name="GOOGLE_SEARCH_API_KEY"
                            type="password"
                            defaultValue={
                              configurations.find(
                                (c) => c.key === "GOOGLE_SEARCH_API_KEY",
                              )?.value || ""
                            }
                            placeholder="AIza..."
                          />
                        </Flex>
                        <Flex direction="column" gap="1">
                          <Text as="label" size="2" weight="bold">
                            Search engine ID
                          </Text>
                          <TextField.Root
                            name="GOOGLE_SEARCH_CX"
                            defaultValue={
                              configurations.find(
                                (c) => c.key === "GOOGLE_SEARCH_CX",
                              )?.value || ""
                            }
                            placeholder="0123456789..."
                          />
                        </Flex>
                      </div>
                    </section>

                    <div className="app-settings-form-footer">
                      <span>Changes apply to future lookups.</span>
                      <Button
                        color="gray"
                        className="app-primary-action"
                        type="submit"
                      >
                        Save changes
                      </Button>
                    </div>
                  </div>
                </form>
              </div>

              <section className="app-settings-panel app-settings-mcp-panel">
                <Flex justify="between" align="start" gap="4" wrap="wrap">
                  <div className="app-settings-panel-heading">
                    <span className="app-settings-panel-icon">
                      <KeyRound size={17} />
                    </span>
                    <div>
                      <h3>MCP access tokens</h3>
                      <p>
                        Let connected LLM clients search and manage saved words.
                        Tokens are shown only once.
                      </p>
                    </div>
                  </div>
                  <Button
                    color="gray"
                    className="app-primary-action"
                    onClick={() => {
                      setTokenName("");
                      setTokenExpiry("90");
                      setTokenScopes({ read: true, write: true });
                      setTokenDialogOpen(true);
                    }}
                  >
                    Generate token
                  </Button>
                </Flex>

                <Flex direction="column" gap="3" mt="5">
                  {mcpTokens.length === 0 ? (
                    <div className="app-settings-empty-row">
                      <KeyRound size={17} />
                      <span>No MCP tokens have been generated.</span>
                    </div>
                  ) : (
                    mcpTokens.map((token) => (
                      <Flex
                        key={token.id}
                        justify="between"
                        align="start"
                        gap="4"
                        className="border-t border-[var(--app-line)] pt-3"
                        wrap="wrap"
                      >
                        <Box>
                          <Text weight="bold" as="div">
                            {token.name}
                          </Text>
                          <Text size="1" color="gray" as="div">
                            {token.scopes.join(", ")} · expires{" "}
                            {formatTokenDate(token.expires_at)}
                          </Text>
                          <Text size="1" color="gray" as="div">
                            Last used: {formatTokenDate(token.last_used_at)}
                            {token.revoked_at ? " · Revoked" : ""}
                          </Text>
                        </Box>
                        <Button
                          size="1"
                          color="red"
                          variant="soft"
                          disabled={!!token.revoked_at}
                          onClick={() => setRevokeTokenId(token.id)}
                        >
                          Revoke
                        </Button>
                      </Flex>
                    ))
                  )}
                </Flex>
              </section>
            </div>
          )}
        </section>
      </div>

      <Dialog.Root
        open={isOpen}
        onOpenChange={(open) => {
          if (!open) {
            setCodexLogin(null);
            setCodexCodeCopied(false);
          }
          onOpenChange(open);
        }}
      >
        <Dialog.Content maxWidth="450px">
          <form onSubmit={handleSave}>
            <Dialog.Title>
              {editingProvider ? "Edit Provider" : "Add Provider"}
            </Dialog.Title>
            <Flex direction="column" gap="3" mt="4">
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Name
                </Text>
                <TextField.Root
                  name="name"
                  defaultValue={editingProvider?.name}
                  readOnly={!!editingProvider}
                  required
                />
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Provider Type
                </Text>
                <Box className="relative">
                  <select
                    name="provider"
                    value={selectedProviderType}
                    onChange={(event) => {
                      setCodexLogin(null);
                      setCodexCodeCopied(false);
                      setSelectedProviderType(event.target.value);
                    }}
                    className="app-native-select w-full pr-10"
                  >
                    <option value="openai">OpenAI</option>
                    <option value="gemini">Gemini</option>
                    <option value="vertexai">VertexAI</option>
                    <option value="workersai">Workers AI</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="codex">OpenAI Codex (ChatGPT)</option>
                  </select>
                  <span className="pointer-events-none absolute inset-y-0 right-4 flex items-center text-[var(--app-muted)]">
                    ▾
                  </span>
                </Box>
              </Flex>
              {selectedProviderType === "codex" ? (
                <Box className="app-codex-login-panel">
                  <input
                    type="hidden"
                    name="base_url"
                    value="https://chatgpt.com/backend-api"
                    readOnly
                  />
                  <input type="hidden" name="api_key" value="" readOnly />
                  <Flex
                    align="start"
                    gap="3"
                    className="app-codex-login-header"
                  >
                    <span className="app-codex-login-icon" aria-hidden="true">
                      <ShieldCheck size={18} />
                    </span>
                    <Box>
                      <Text size="3" weight="bold" as="div">
                        Sign in with ChatGPT
                      </Text>
                      <Text size="2" color="gray" as="p" mt="1">
                        Use your ChatGPT subscription with Codex. No API key is
                        required.
                      </Text>
                    </Box>
                  </Flex>
                  {codexLogin ? (
                    <Box className="app-codex-login-flow">
                      <Flex
                        align="center"
                        gap="2"
                        className="app-codex-login-step"
                      >
                        <span className="app-codex-step-number">1</span>
                        <Text size="2">
                          Open OpenAI&apos;s device sign-in page.
                        </Text>
                      </Flex>
                      <a
                        href={codexLogin.verification_url}
                        target="_blank"
                        rel="noreferrer"
                        className="app-codex-login-action"
                      >
                        <span className="app-codex-login-action-content">
                          <ExternalLink size={16} aria-hidden="true" />
                          <span>Open OpenAI device sign-in</span>
                        </span>
                        <ArrowUpRight
                          size={16}
                          aria-hidden="true"
                          className="app-codex-login-action-arrow"
                        />
                      </a>
                      <Flex
                        align="end"
                        justify="between"
                        gap="3"
                        className="app-codex-code-row"
                      >
                        <Box className="app-codex-code-block">
                          <Text size="1" color="gray" as="div">
                            2 · Enter this device code
                          </Text>
                          <Text
                            size="6"
                            weight="bold"
                            className="app-codex-code"
                          >
                            {codexLogin.user_code}
                          </Text>
                        </Box>
                        <Button
                          type="button"
                          size="1"
                          variant="soft"
                          color="gray"
                          className="app-codex-copy-button"
                          onClick={() => void handleCopyCodexCode()}
                        >
                          {codexCodeCopied ? (
                            <Check size={15} aria-hidden="true" />
                          ) : (
                            <Copy size={15} aria-hidden="true" />
                          )}
                          {codexCodeCopied ? "Copied" : "Copy code"}
                        </Button>
                      </Flex>
                      <Flex
                        align="center"
                        gap="2"
                        role="status"
                        className="app-codex-login-status"
                      >
                        <Spinner size="1" />
                        <Text size="1" color="gray">
                          Waiting for authorization…
                        </Text>
                      </Flex>
                    </Box>
                  ) : (
                    <Text size="2" color="gray" as="p" mt="4">
                      Click &quot;Sign in with ChatGPT&quot; below to get a
                      one-time device code.
                    </Text>
                  )}
                </Box>
              ) : (
                <>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      Base URL (Optional)
                    </Text>
                    <TextField.Root
                      name="base_url"
                      defaultValue={editingProvider?.base_url}
                    />
                  </Flex>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      API Key
                    </Text>
                    <TextField.Root
                      name="api_key"
                      type="password"
                      defaultValue={editingProvider?.api_key}
                    />
                  </Flex>
                </>
              )}
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Models (comma separated)
                </Text>
                <TextField.Root
                  name="models"
                  defaultValue={editingProvider?.models}
                  placeholder={
                    selectedProviderType === "codex" ? "gpt-5.4" : undefined
                  }
                  required
                />
                {selectedProviderType === "codex" && (
                  <Text size="1" color="gray">
                    Enter one or more Codex model ids, separated by commas.
                  </Text>
                )}
              </Flex>
              {selectedProviderType === "anthropic" && (
                <>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      Anthropic Version
                    </Text>
                    <TextField.Root
                      name="anthropic_version"
                      value={anthropicVersion}
                      onChange={(e) => setAnthropicVersion(e.target.value)}
                    />
                  </Flex>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      Extended Thinking
                    </Text>
                    <Flex align="center" gap="2">
                      <Switch
                        id="thinking_enabled"
                        checked={thinkingEnabled}
                        onCheckedChange={setThinkingEnabled}
                      />
                      <Text
                        as="label"
                        size="2"
                        htmlFor="thinking_enabled"
                        className="cursor-pointer"
                      >
                        Enable Extended Thinking
                      </Text>
                    </Flex>
                    {thinkingEnabled && (
                      <Flex direction="column" gap="1" mt="2">
                        <Text as="label" size="2" weight="bold">
                          Budget Tokens (min 1024)
                        </Text>
                        <TextField.Root
                          type="number"
                          value={thinkingBudget}
                          onChange={(e) => setThinkingBudget(e.target.value)}
                          min="1024"
                        />
                      </Flex>
                    )}
                  </Flex>
                </>
              )}
              {selectedProviderType === "vertexai" && (
                <>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      Project ID (for VertexAI)
                    </Text>
                    <TextField.Root
                      name="project_id"
                      defaultValue={projectId}
                    />
                  </Flex>
                  <Flex direction="column" gap="1">
                    <Text as="label" size="2" weight="bold">
                      Location (for VertexAI)
                    </Text>
                    <TextField.Root
                      name="location"
                      defaultValue={vertexLocation}
                    />
                  </Flex>
                </>
              )}
              {(selectedProviderType === "gemini" ||
                selectedProviderType === "vertexai") && (
                <Flex direction="column" gap="1">
                  <Text as="label" size="2" weight="bold">
                    Features
                  </Text>
                  <Flex align="center" gap="2">
                    <Switch
                      id="gemini_search"
                      checked={geminiSearch}
                      onCheckedChange={setGeminiSearch}
                    />
                    <Text
                      as="label"
                      size="2"
                      htmlFor="gemini_search"
                      className="cursor-pointer"
                    >
                      Enable Gemini Google Search Grounding
                    </Text>
                  </Flex>
                  {geminiSearch && (
                    <Text size="1" color="orange" className="block mt-1">
                      Warning: Gemini models charge per search query executed.
                    </Text>
                  )}
                </Flex>
              )}
            </Flex>
            <Flex gap="3" mt="4" justify="end">
              <Button
                color="gray"
                variant="soft"
                className="app-secondary-action"
                type="button"
                onClick={() => {
                  setCodexLogin(null);
                  setCodexCodeCopied(false);
                  onOpenChange(false);
                }}
              >
                Cancel
              </Button>
              <Button
                color="gray"
                className="app-primary-action"
                type="submit"
                disabled={codexLoginLoading || !!codexLogin}
              >
                {selectedProviderType === "codex" &&
                !isCodexConnected(editingProvider)
                  ? codexLogin
                    ? "Waiting for ChatGPT…"
                    : codexLoginLoading
                      ? "Starting sign-in…"
                      : "Sign in with ChatGPT"
                  : "Save"}
              </Button>
            </Flex>
          </form>
        </Dialog.Content>
      </Dialog.Root>

      <Dialog.Root open={tokenDialogOpen} onOpenChange={setTokenDialogOpen}>
        <Dialog.Content maxWidth="450px">
          <form onSubmit={handleCreateMcpToken}>
            <Dialog.Title>Generate MCP token</Dialog.Title>
            <Text size="2" color="gray" as="p" mt="2">
              Store this token in the LLM client configuration. It will not be
              shown again after this dialog is closed.
            </Text>
            <Flex direction="column" gap="3" mt="4">
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Name
                </Text>
                <TextField.Root
                  value={tokenName}
                  onChange={(event) => setTokenName(event.target.value)}
                  placeholder="Claude dictionary assistant"
                  required
                />
              </Flex>
              <Flex direction="column" gap="2">
                <Text as="label" size="2" weight="bold">
                  Permissions
                </Text>
                <Flex align="center" gap="2">
                  <Switch
                    checked={tokenScopes.read}
                    onCheckedChange={(checked) =>
                      setTokenScopes((current) => ({
                        ...current,
                        read: checked,
                      }))
                    }
                  />
                  <Text size="2">dictionary:read</Text>
                </Flex>
                <Flex align="center" gap="2">
                  <Switch
                    checked={tokenScopes.write}
                    onCheckedChange={(checked) =>
                      setTokenScopes((current) => ({
                        ...current,
                        write: checked,
                      }))
                    }
                  />
                  <Text size="2">dictionary:write</Text>
                </Flex>
              </Flex>
              <Flex direction="column" gap="1">
                <Text as="label" size="2" weight="bold">
                  Expiration
                </Text>
                <select
                  value={tokenExpiry}
                  onChange={(event) => setTokenExpiry(event.target.value)}
                  className="app-native-select w-full"
                >
                  <option value="30">30 days</option>
                  <option value="90">90 days</option>
                  <option value="365">365 days</option>
                  <option value="never">Never</option>
                </select>
              </Flex>
            </Flex>
            <Flex gap="3" mt="5" justify="end">
              <Button
                color="gray"
                variant="soft"
                type="button"
                onClick={() => setTokenDialogOpen(false)}
              >
                Cancel
              </Button>
              <Button color="gray" className="app-primary-action" type="submit">
                Generate
              </Button>
            </Flex>
          </form>
        </Dialog.Content>
      </Dialog.Root>

      <Dialog.Root
        open={!!createdToken}
        onOpenChange={(open) => {
          if (!open) setCreatedToken(null);
        }}
      >
        <Dialog.Content maxWidth="520px">
          <Dialog.Title>Token generated</Dialog.Title>
          <Text size="2" color="gray" as="p" mt="2">
            Copy this token now. For security, it is stored as a hash and will
            not be displayed again.
          </Text>
          {createdToken && (
            <>
              <Box
                mt="4"
                p="3"
                className="break-all rounded-md border border-[var(--app-line)] bg-[var(--app-subtle)] font-mono text-sm"
              >
                {createdToken.token}
              </Box>
              <Flex justify="between" align="center" mt="3" gap="3" wrap="wrap">
                <Text size="1" color="gray">
                  {createdToken.scopes.join(", ")} · expires{" "}
                  {formatTokenDate(createdToken.expires_at)}
                </Text>
                <Button
                  color="gray"
                  className="app-primary-action"
                  onClick={() => {
                    void navigator.clipboard.writeText(createdToken.token);
                    addToast({ title: "Token copied", color: "success" });
                  }}
                >
                  Copy token
                </Button>
              </Flex>
            </>
          )}
        </Dialog.Content>
      </Dialog.Root>
    </PageContainer>
  );
}
