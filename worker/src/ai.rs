use hjdef::ai::AI;

pub struct WasmAI {}

impl Clone for WasmAI {
    fn clone(&self) -> Self {
        Self {}
    }
}

impl AI for WasmAI {
    async fn gemma3_12b(&self, _prompt: String) -> Result<String, hjdef::ai::Error> {
        Ok("".to_string())
    }

    async fn llama4_scout_17b_16e_instruct(
        &self,
        _prompt: String,
    ) -> Result<String, hjdef::ai::Error> {
        Ok("".to_string())
    }

    async fn m2m100_1_2b(
        &self,
        _text: &str,
        _source_lang: &str,
        _target_lang: &str,
    ) -> Result<String, hjdef::ai::Error> {
        Ok("".to_string())
    }
}
