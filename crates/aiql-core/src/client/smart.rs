use crate::{ExecutionEngine, ExecutionResult, QueryHealer, Schema, Translator};

pub struct SmartClient<T, E, H>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
{
    translator: T,
    engine: E,
    healer: H,
}

impl<T, E, H> SmartClient<T, E, H>
where
    T: Translator,
    E: ExecutionEngine,
    H: QueryHealer,
{
    pub fn new(translator: T, engine: E, healer: H) -> Self {
        Self {
            translator,
            engine,
            healer,
        }
    }

    pub async fn ask(&self, prompt: &str, schema: &Schema) -> anyhow::Result<ExecutionResult> {
        // 1. Translate
        let plan = self.translator.translate(prompt, schema).await?;

        // 2. Dry Run
        if !self.engine.dry_run(&plan.raw_query).await? {
             return Err(anyhow::anyhow!("Dry run failed for generated query"));
        }

        // 3. Execute
        let mut result = self.engine.execute(&plan.raw_query).await?;

        // 4. Self-Healing Loop
        if !result.success {
            if let Some(error_msg) = result.error.clone() {
                // 5. Heal
                let healed_plan = self.healer.heal(&plan.raw_query, &error_msg, schema).await?;
                
                // 6. Retry
                result = self.engine.execute(&healed_plan.raw_query).await?;
            }
        }

        Ok(result)
    }
}
