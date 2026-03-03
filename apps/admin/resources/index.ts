import { usersResource } from "./users";
import { blogsResource } from "./blogs";
import { executionLogsResource } from "./execution-logs";
import { endpointsResource } from "./endpoints";
// grit:resources

import type { ResourceDefinition } from "@/lib/resource";

export const resources: ResourceDefinition[] = [
  usersResource,
  blogsResource,
  executionLogsResource,
  endpointsResource,
  // grit:resource-list
];

export function getResource(slug: string): ResourceDefinition | undefined {
  return resources.find((r) => r.slug === slug);
}

export function getResourceByEndpoint(endpoint: string): ResourceDefinition | undefined {
  return resources.find((r) => r.endpoint === endpoint);
}
