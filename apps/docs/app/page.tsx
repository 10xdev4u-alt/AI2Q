import Link from "next/link";

export default function HomePage() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-fd-background text-fd-foreground">
      <div className="text-center max-w-2xl px-6">
        <h1 className="text-5xl font-bold mb-4">
          Documentation
        </h1>
        <p className="text-lg text-fd-muted-foreground mb-8">
          Everything you need to build with Grit — the full-stack Go + React framework.
        </p>
        <Link
          href="/docs"
          className="inline-flex items-center px-6 py-3 rounded-lg bg-fd-primary text-fd-primary-foreground font-medium hover:bg-fd-primary/90 transition-colors"
        >
          Get Started
        </Link>
      </div>
    </main>
  );
}
