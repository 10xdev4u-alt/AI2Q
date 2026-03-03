"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent } from "@/components/ui/card"
import { Terminal, Send, Zap, Shield, RefreshCw } from "lucide-react"

export default function PlaygroundPage() {
  const [prompt, setPrompt] = useState("")
  const [history, setHistory] = useState([
    { role: "system", content: "AIQL Engine v1.0.0 Online. Standing by for natural language instructions." }
  ])
  const [isQuerying, setIsQuerying] = useState(false)

  const handleQuery = async () => {
    if (!prompt) return
    setIsQuerying(true)
    setHistory(prev => [...prev, { role: "user", content: prompt }])
    
    // Simulate AIQL Core response
    setTimeout(() => {
      setHistory(prev => [...prev, { 
        role: "ai", 
        content: `SELECT * FROM users WHERE created_at >= NOW() - INTERVAL '7 days' LIMIT 10;`,
        explanation: "Filtering users who signed up in the last week."
      }])
      setIsQuerying(false)
      setPrompt("")
    }, 1500)
  }

  return (
    <div className="min-h-screen bg-background p-8 font-mono">
      <div className="max-w-5xl mx-auto space-y-8">
        <div className="border-b-8 border-foreground pb-6">
          <h1 className="text-5xl font-black uppercase tracking-tighter">Brutalist Playground</h1>
          <p className="text-xl font-bold mt-2 opacity-70 italic underline">Deep-dive into the AIQL translation engine.</p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* Sidebar - Engine Stats */}
          <div className="lg:col-span-1 space-y-6">
            <Card className="border-4 border-foreground rounded-none shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] bg-primary/10">
              <CardContent className="p-4 space-y-4">
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2">Engine Status</div>
                <div className="flex items-center gap-2 font-bold text-green-600 animate-pulse">
                  <Zap className="w-4 h-4" /> ACTIVE
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2">Security</div>
                <div className="flex items-center gap-2 font-bold text-blue-600">
                  <Shield className="w-4 h-4" /> PII_SCRUB_ON
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2">Cache</div>
                <div className="flex items-center gap-2 font-bold text-purple-600">
                  <RefreshCw className="w-4 h-4" /> SEMANTIC_HIT_Ready
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Main Terminal */}
          <div className="lg:col-span-3 flex flex-col h-[70vh] border-8 border-foreground shadow-[12px_12px_0px_0px_rgba(0,0,0,1)] bg-zinc-950 overflow-hidden">
            <div className="bg-foreground text-background p-2 px-4 flex items-center justify-between">
              <div className="flex items-center gap-2 text-sm font-black uppercase">
                <Terminal className="w-4 h-4" /> aiql-core-debug-v1
              </div>
              <div className="flex gap-2">
                <div className="w-3 h-3 bg-red-500 rounded-full" />
                <div className="w-3 h-3 bg-yellow-500 rounded-full" />
                <div className="w-3 h-3 bg-green-500 rounded-full" />
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-6 space-y-6">
              {history.map((msg, i) => (
                <div key={i} className={`space-y-2 ${msg.role === 'user' ? 'text-blue-400' : msg.role === 'system' ? 'text-zinc-500 italic' : 'text-green-400'}`}>
                  <div className="flex items-start gap-3">
                    <span className="font-black uppercase min-w-[80px]">{msg.role === 'user' ? '>>>' : msg.role === 'system' ? '[SYS]' : 'AIQL'}</span>
                    <div className="flex-1 whitespace-pre-wrap font-bold leading-tight">
                      {msg.content}
                      {msg.explanation && (
                        <div className="mt-4 p-4 bg-zinc-900 border-2 border-dashed border-zinc-700 text-zinc-400 text-sm flex justify-between items-end">
                          <div>-- {msg.explanation}</div>
                          {msg.role === 'ai' && (
                            <Button variant="outline" size="sm" className="h-8 border-zinc-700 text-xs font-black uppercase hover:bg-zinc-800">
                              Export API
                            </Button>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
              {isQuerying && (
                <div className="flex items-center gap-3 text-yellow-400 animate-pulse font-black uppercase">
                  <span>TRANSLATING</span>
                  <div className="flex gap-1">
                    <div className="w-2 h-2 bg-yellow-400 animate-bounce" />
                    <div className="w-2 h-2 bg-yellow-400 animate-bounce [animation-delay:0.2s]" />
                    <div className="w-2 h-2 bg-yellow-400 animate-bounce [animation-delay:0.4s]" />
                  </div>
                </div>
              )}
            </div>

            <div className="p-4 border-t-4 border-foreground bg-zinc-900 flex gap-4">
              <Input 
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleQuery()}
                placeholder="Ask your database anything..."
                className="flex-1 h-14 bg-zinc-800 border-2 border-foreground text-xl font-bold rounded-none focus-visible:ring-0 focus-visible:border-primary text-white"
              />
              <Button 
                onClick={handleQuery}
                disabled={isQuerying}
                className="h-14 px-8 rounded-none border-2 border-foreground shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-1 hover:translate-y-1 transition-all"
              >
                <Send className="w-6 h-6" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
