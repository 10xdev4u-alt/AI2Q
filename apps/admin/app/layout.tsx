import type { Metadata } from "next";
import { Onest, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/shared/providers";

const onest = Onest({
  subsets: ["latin"],
  variable: "--font-onest",
  weight: ["400", "500", "600", "700"],
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono",
  weight: ["400", "500", "600"],
});

export const metadata: Metadata = {
  title: "webapp Admin",
  description: "Admin panel — Built with Grit",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className={`${onest.variable} ${jetbrainsMono.variable} min-h-screen bg-background font-sans antialiased`}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
