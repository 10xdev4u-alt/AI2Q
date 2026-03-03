import "./global.css";
import { RootProvider } from "fumadocs-ui/provider";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: {
    template: "%s | webapp Docs",
    default: "webapp Documentation",
  },
  description: "Documentation for webapp — Go + React. Built with Grit.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body>
        <RootProvider
          theme={{
            enabled: true,
            defaultTheme: "dark",
          }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
