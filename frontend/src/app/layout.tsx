import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { Navbar } from "@/components/navbar";
import { Footer } from "@/components/footer";
import { ParticleBackground } from "@/components/particle-background";

const inter = Inter({
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Privy SVM",
  description:
    "Privacy-preserving zero-knowledge proofs on Solana. Secure, trustless, and verifiable computation powered by ZK-SNARKs.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${inter.className} bg-neutral-950 text-white min-h-screen flex flex-col`}
      >
        <ParticleBackground />
        <Navbar />
        <main className="relative z-10 flex-1">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
