"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Database, Search, Shield, Zap, RefreshCw, Layers } from "lucide-react"
import Link from "next/link"

export default function Home() {
  const [demoPrompt, setDemoPrompt] = useState("")
  const [demoOutput, setDemoOutput] = useState("")

  const handleDemo = () => {
    if (!demoPrompt) return
    setDemoOutput("TRANSLATING...")
    setTimeout(() => {
      setDemoOutput("SELECT * FROM users\nWHERE created_at >= NOW() - INTERVAL '7 days';")
    }, 1000)
  }

  return (
    <div className="flex flex-col min-h-screen bg-background text-foreground selection:bg-primary selection:text-primary-foreground font-mono">
      {/* Hero Section */}
      <section className="py-20 px-6 border-b-4 border-foreground">
        <div className="max-w-6xl mx-auto space-y-12">
          <div className="space-y-6">
            <Badge variant="outline" className="border-2 border-foreground rounded-none px-4 py-1 text-lg font-bold">
              v1.0.0-ALPHA
            </Badge>
            <h1 className="text-7xl md:text-9xl font-black tracking-tighter uppercase leading-none">
              AI<span className="text-primary italic underline underline-offset-8 decoration-4">QL</span>
            </h1>
            <p className="text-2xl md:text-4xl font-bold max-w-3xl leading-tight border-l-8 border-foreground pl-6 py-4">
              Don't Query. Just Ask. The Universal AI Intelligence Driver for Your Database.
            </p>
          </div>

          <div className="flex flex-col md:flex-row gap-6">
            <Link href="/playground">
              <Button size="lg" className="h-16 px-12 text-2xl font-black rounded-none border-4 border-foreground shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-[4px] hover:translate-y-[4px] transition-all w-full md:w-auto">
                GET STARTED
              </Button>
            </Link>
            <Link href="/schema">
              <Button size="lg" variant="outline" className="h-16 px-12 text-2xl font-black rounded-none border-4 border-foreground shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] bg-white text-black hover:shadow-none hover:translate-x-[4px] hover:translate-y-[4px] transition-all w-full md:w-auto">
                VIEW SCHEMA
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Feature Grid */}
      <section className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        {[
          {
            title: "Schema Crawler",
            desc: "Auto-ingests your entire database schema with PK/FK and Index detection.",
            icon: Database,
            color: "bg-blue-400"
          },
          {
            title: "Self-Healing",
            desc: "Invisibly retries queries on schema errors, correcting hallucinations automatically.",
            icon: RefreshCw,
            color: "bg-green-400"
          },
          {
            title: "Universal Bridge",
            desc: "Write once in Go, Python, or C++ using our shared Rust core.",
            icon: Layers,
            color: "bg-purple-400"
          },
          {
            title: "Ambiguity Guard",
            desc: "No more guessing. AIQL asks for clarification when prompts are too vague.",
            icon: Search,
            color: "bg-yellow-400"
          },
          {
            title: "Security First",
            desc: "PII masking and read-only enforcement built into the driver core.",
            icon: Shield,
            color: "bg-red-400"
          },
          {
            title: "High Performance",
            desc: "Rust-powered engine for ultra-low latency Txt2Sql translation.",
            icon: Zap,
            color: "bg-orange-400"
          }
        ].map((feature, i) => (
          <div key={i} className={`p-10 border-r-4 border-b-4 border-foreground group hover:${feature.color} transition-colors duration-500`}>
            <div className="mb-6">
              <feature.icon className="w-16 h-16 stroke-[3px]" />
            </div>
            <h3 className="text-3xl font-black mb-4 uppercase">{feature.title}</h3>
            <p className="text-xl font-bold leading-snug">
              {feature.desc}
            </p>
          </div>
        ))}
      </section>

      {/* Console Playground */}
      <section className="py-20 px-6 border-b-4 border-foreground bg-foreground text-background">
        <div className="max-w-4xl mx-auto space-y-12">
          <div className="text-center space-y-4">
            <h2 className="text-5xl md:text-7xl font-black uppercase tracking-tighter italic">Experience the Magic</h2>
            <p className="text-xl md:text-2xl font-bold opacity-80">Try the natural language interface in our playground</p>
          </div>

          <div className="border-8 border-background bg-zinc-900 p-8 shadow-[12px_12px_0px_0px_rgba(255,255,255,0.2)]">
            <div className="flex gap-4 mb-8">
              <Input 
                value={demoPrompt}
                onChange={(e) => setDemoPrompt(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleDemo()}
                placeholder="Ask your database: 'Top 5 users by revenue this month...'" 
                className="h-16 bg-zinc-800 border-4 border-background text-2xl font-bold rounded-none focus-visible:ring-0 focus-visible:border-primary text-white"
              />
              <Button onClick={handleDemo} className="h-16 px-8 text-2xl font-black rounded-none border-4 border-background bg-primary hover:bg-primary/90">
                QUERY
              </Button>
            </div>

            <div className="space-y-4 font-mono text-xl opacity-80 text-white min-h-[100px]">
              {demoOutput && (
                <div className="p-6 bg-zinc-800 border-2 border-dashed border-background/20 animate-in fade-in slide-in-from-top-2">
                  <pre className="text-yellow-400 whitespace-pre-wrap">
                    {demoOutput}
                  </pre>
                </div>
              )}
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}
