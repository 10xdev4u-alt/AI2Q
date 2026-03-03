use crate::{Exporter, QueryPlan};

pub struct SupabaseExporter;

impl Exporter for SupabaseExporter {
    fn export(&self, plan: &QueryPlan) -> anyhow::Result<String> {
        let sql = &plan.raw_query;
        let explanation = &plan.explanation;

        let code = format!(
            r#"// Supabase Edge Function: AIQL Export
// Prompt Explanation: {}

import {{ serve }} from "https://deno.land/std@0.168.0/http/server.ts"
import {{ createClient }} from "https://esm.sh/@supabase/supabase-js@2"

serve(async (req) => {{
  const supabase = createClient(
    Deno.env.get('SUPABASE_URL') ?? '',
    Deno.env.get('SUPABASE_SERVICE_ROLE_KEY') ?? ''
  )

  const {{ data, error }} = await supabase
    .rpc('execute_ai_query', {{ query_text: `{}` }})

  if (error) return new Response(JSON.stringify({{ error: error.message }}), {{ status: 500 }})

  return new Response(
    JSON.stringify(data),
    {{ headers: {{ "Content-Type": "application/json" }} }},
  )
}})
"#,
            explanation, sql
        );

        Ok(code)
    }
}
