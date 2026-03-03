import { ResourceDefinition } from "@/lib/resource";
import { Terminal } from "lucide-react";

export const executionLogsResource: ResourceDefinition = {
  name: "Execution Logs",
  slug: "execution-logs",
  endpoint: "/execution-logs",
  icon: Terminal,
  columns: [
    { key: "prompt", label: "Prompt", sortable: true, searchable: true },
    { key: "sql", label: "SQL", sortable: false },
    { key: "execution_time", label: "Time (ms)", sortable: true },
    { key: "success", label: "Success", sortable: true, type: "boolean" },
    { key: "dialect", label: "Dialect", sortable: true },
  ],
  fields: [
    { key: "prompt", label: "Prompt", type: "textarea", required: true },
    { key: "sql", label: "SQL", type: "textarea", required: true },
    { key: "execution_time", label: "Execution Time", type: "number" },
    { key: "success", label: "Success", type: "toggle" },
    { key: "error", label: "Error", type: "textarea" },
    { key: "dialect", label: "Dialect", type: "text" },
  ],
};
