"use client";

import { useMe } from "@/hooks/use-auth";
import { resources } from "@/resources";
import { StatsCard } from "@/components/widgets/stats-card";
import { WidgetGrid } from "@/components/widgets/widget-grid";
import { getIcon } from "@/lib/icons";

export default function AdminDashboard() {
  const { data: user } = useMe();
  const allWidgets = resources.flatMap((r) => r.dashboard?.widgets ?? []);

  const greeting = () => {
    const hour = new Date().getHours();
    if (hour < 12) return "Good morning";
    if (hour < 17) return "Good afternoon";
    return "Good evening";
  };

  return (
    <div className="space-y-8">
      {/* Welcome header */}
      <div className="rounded-xl border border-border bg-gradient-to-r from-accent/10 via-bg-secondary to-bg-secondary p-6 sm:p-8">
        <h1 className="text-2xl font-bold text-foreground">
          {greeting()}, {user?.first_name || "Admin"}
        </h1>
        <p className="text-text-secondary mt-1">
          Here&apos;s an overview of your application.
        </p>
      </div>

      {/* Stats widgets */}
      {allWidgets.length > 0 ? (
        <WidgetGrid widgets={allWidgets} />
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatsCard label="Total Resources" value="—" icon="Database" color="accent" />
          <StatsCard label="Registered" value={String(resources.length)} icon="Layers" color="success" />
        </div>
      )}

      {/* Quick Actions + System */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Resources */}
        <div className="lg:col-span-2 rounded-xl border border-border bg-bg-secondary p-6">
          <h2 className="text-lg font-semibold text-foreground mb-4">Resources</h2>
          <div className="grid gap-3 sm:grid-cols-2">
            {resources.map((r) => {
              const Icon = getIcon(r.icon);
              return (
                <a
                  key={r.slug}
                  href={`/resources/${r.slug}`}
                  className="flex items-center gap-4 rounded-lg border border-border bg-bg-tertiary p-4 hover:border-accent/30 hover:bg-bg-hover transition-all group"
                >
                  <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-accent/10 group-hover:bg-accent/20 transition-colors">
                    <Icon className="h-5 w-5 text-accent" />
                  </div>
                  <div>
                    <h3 className="font-medium text-foreground group-hover:text-accent transition-colors">
                      {r.label?.plural ?? r.name}
                    </h3>
                    <p className="text-xs text-text-muted">
                      Manage {(r.label?.plural ?? r.slug).toLowerCase()}
                    </p>
                  </div>
                </a>
              );
            })}
          </div>
        </div>

        {/* Quick Links */}
        <div className="rounded-xl border border-border bg-bg-secondary p-6">
          <h2 className="text-lg font-semibold text-foreground mb-4">Quick Links</h2>
          <div className="space-y-2">
            <a
              href="http://localhost:8080/studio"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-3 rounded-lg border border-border bg-bg-tertiary px-4 py-3 hover:border-accent/30 hover:bg-bg-hover transition-all group"
            >
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-info/10">
                <span className="text-info text-sm font-bold">DB</span>
              </div>
              <div>
                <p className="text-sm font-medium text-foreground group-hover:text-accent transition-colors">GORM Studio</p>
                <p className="text-xs text-text-muted">Browse database</p>
              </div>
            </a>
            <a
              href="http://localhost:8080/api/health"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-3 rounded-lg border border-border bg-bg-tertiary px-4 py-3 hover:border-accent/30 hover:bg-bg-hover transition-all group"
            >
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-success/10">
                <span className="text-success text-sm font-bold">OK</span>
              </div>
              <div>
                <p className="text-sm font-medium text-foreground group-hover:text-accent transition-colors">API Health</p>
                <p className="text-xs text-text-muted">Check status</p>
              </div>
            </a>
            <a
              href="/system/jobs"
              className="flex items-center gap-3 rounded-lg border border-border bg-bg-tertiary px-4 py-3 hover:border-accent/30 hover:bg-bg-hover transition-all group"
            >
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-warning/10">
                <span className="text-warning text-sm font-bold">Q</span>
              </div>
              <div>
                <p className="text-sm font-medium text-foreground group-hover:text-accent transition-colors">Job Queue</p>
                <p className="text-xs text-text-muted">Background jobs</p>
              </div>
            </a>
            <a
              href="/system/files"
              className="flex items-center gap-3 rounded-lg border border-border bg-bg-tertiary px-4 py-3 hover:border-accent/30 hover:bg-bg-hover transition-all group"
            >
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent/10">
                <span className="text-accent text-sm font-bold">S3</span>
              </div>
              <div>
                <p className="text-sm font-medium text-foreground group-hover:text-accent transition-colors">File Storage</p>
                <p className="text-xs text-text-muted">Manage uploads</p>
              </div>
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
