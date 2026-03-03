import { api } from "./api";

export interface Column {
  name: string;
  data_type: string;
  is_nullable: boolean;
  is_primary_key: boolean;
  default_value: string | null;
  description: string | null;
}

export interface ForeignKey {
  constraint_name: string;
  column_name: string;
  foreign_table: string;
  foreign_column: string;
}

export interface Table {
  name: string;
  columns: Column[];
  foreign_keys: ForeignKey[];
  indexes: any[];
}

export interface Schema {
  version: string;
  created_at: string;
  tables: Record<string, Table>;
}

export interface TranslateResult {
  type: "plan" | "clarification";
  dialect?: string;
  raw_query?: string;
  explanation?: string;
  reason?: string;
  suggestions?: string[];
}

export const aiqlApi = {
  crawl: async (url: string): Promise<Schema> => {
    const { data } = await api.post("/api/aiql/crawl", { url });
    return data;
  },
  translate: async (prompt: string, schema: Schema): Promise<TranslateResult> => {
    const { data } = await api.post("/api/aiql/translate", { 
      prompt, 
      schema_json: JSON.stringify(schema) 
    });
    return data;
  },
  export: async (name: string, path: string, prompt: string, sql: string): Promise<any> => {
    const { data } = await api.post("/api/endpoints", {
      name,
      path,
      prompt,
      sql,
      method: "GET"
    });
    return data;
  }
};
