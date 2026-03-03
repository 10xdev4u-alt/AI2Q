import { ResourceDefinition } from "@/lib/resource";
import { Link } from "lucide-react";

export const endpointsResource: ResourceDefinition = {
  name: "API Endpoints",
  slug: "endpoints",
  endpoint: "/endpoints",
  icon: Link,
  columns: [
    { key: "name", label: "Name", sortable: true, searchable: true },
    { key: "path", label: "Path", sortable: true },
    { key: "method", label: "Method", sortable: true },
  ],
  fields: [
    { key: "name", label: "Name", type: "text", required: true },
    { key: "path", label: "Path", type: "text", required: true },
    { key: "prompt", label: "Original Prompt", type: "textarea", required: true },
    { key: "sql", label: "SQL Query", type: "textarea", required: true },
    { key: "description", label: "Description", type: "textarea" },
    { key: "method", label: "HTTP Method", type: "select", options: [
      { label: "GET", value: "GET" },
      { label: "POST", value: "POST" }
    ], defaultValue: "GET" },
  ],
};
