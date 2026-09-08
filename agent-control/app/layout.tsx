import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Agent Control — Mission control",
  description: "Observe and control your issue implementation agents.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><body>{children}</body></html>;
}
