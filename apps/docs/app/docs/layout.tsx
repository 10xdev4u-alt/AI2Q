import { DocsLayout } from "fumadocs-ui/layouts/docs";
import type { ReactNode } from "react";
import { source } from "@/app/source";

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <DocsLayout
      tree={source.pageTree}
      nav={{
        title: (
          <span className="flex items-center gap-2 font-bold">
            Grit Docs
            <span className="rounded-md bg-fd-primary/10 px-1.5 py-0.5 text-xs font-medium text-fd-primary">
              v0.13.0
            </span>
          </span>
        ),
      }}
    >
      {children}
    </DocsLayout>
  );
}
