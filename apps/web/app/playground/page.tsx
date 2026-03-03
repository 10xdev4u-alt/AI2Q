"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent } from "@/components/ui/card"
import { Terminal, Send, Zap, Shield, RefreshCw, Loader2 } from "lucide-react"
import { aiqlApi, TranslateResult, Schema } from "@/lib/aiql"
import { Badge } from "@/components/ui/badge"

export default function PlaygroundPage() {
  const [prompt, setPrompt] = useState("")
  const [dialect, setDialect] = useState("postgres")
  const [history, setHistory] = useState<any[]>([
    { role: "system", content: "AIQL Engine v1.0.0 Online. Standing by for natural language instructions." }
  ])
  const [isQuerying, setIsQuerying] = useState(false)

  const handleQuery = async () => {
    if (!prompt) return
    setIsQuerying(true)
    setHistory(prev => [...prev, { role: "user", content: prompt }])
    
    try {
      // Small test schema
      const testSchema: Schema = {
        version: "1.0",
        created_at: new Date().toISOString(),
        tables: {
          "users": {
            name: "users",
            columns: [
              { name: "id", data_type: "uuid", is_primary_key: true, is_nullable: false, default_value: null, description: null },
              { name: "email", data_type: "text", is_primary_key: false, is_nullable: false, default_value: null, description: null },
              { name: "created_at", data_type: "timestamp", is_primary_key: false, is_nullable: false, default_value: null, description: null },
            ],
            foreign_keys: [],
            indexes: []
          }
        }
      }

      const result = await aiqlApi.translate(prompt, testSchema)
      
      if (result.type === "plan") {
        setHistory(prev => [...prev, { 
          role: "ai", 
          content: result.raw_query,
          explanation: result.explanation 
        }])
      } else {
        setHistory(prev => [...prev, { 
          role: "system", 
          content: `Clarification Needed: ${result.reason}`,
          suggestions: result.suggestions
        }])
      }
    } catch (err: any) {
      setHistory(prev => [...prev, { role: "system", content: `Error: ${err.response?.data?.error || err.message}` }])
    } finally {
      setIsQuerying(false)
      setPrompt("")
    }
  }

  const handleExport = async (msgPrompt: string, sql: string) => {
    const name = window.prompt("Enter a name for this API endpoint:", "My AI Query");
    if (!name) return;
    const path = window.prompt("Enter a URL path for this endpoint (e.g., my-query):", name.toLowerCase().replace(/\s+/g, "-"));
    if (!path) return;

    try {
      await aiqlApi.export(name, path, msgPrompt, sql);
      alert(`API Exported successfully! URL: http://localhost:8080/e/${path}`);
    } catch (err: any) {
      alert(`Failed to export API: ${err.message}`);
    }
  }

  return (
    <div className="min-h-screen bg-background p-8 font-mono text-foreground">
      <div className="max-w-5xl mx-auto space-y-8 text-foreground">
        <div className="border-b-8 border-foreground pb-6">
          <h1 className="text-5xl font-black uppercase tracking-tighter text-foreground leading-none">Brutalist Playground</h1>
          <p className="text-xl font-bold mt-2 opacity-70 italic underline">Deep-dive into the AIQL translation engine.</p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          <div className="lg:col-span-1 space-y-6">
            <Card className="border-4 border-foreground rounded-none shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] bg-primary/10">
              <CardContent className="p-4 space-y-4 text-foreground">
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 text-foreground">Target Dialect</div>
                <select 
                  value={dialect} 
                  onChange={(e) => setDialect(e.target.value)}
                  className="w-full bg-background border-2 border-foreground p-2 font-black text-xs uppercase focus:ring-0 outline-none text-foreground"
                >
                  <option value="postgres">Postgres (SQL)</option>
                  <option value="mongodb">MongoDB (MQL)</option>
                  <option value="postgrest">Supabase (JS)</option>
                </select>

                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Engine Status</div>
                <div className="flex items-center gap-2 font-bold text-green-600 animate-pulse">
                  <Zap className="w-4 h-4" /> ACTIVE
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Security</div>
                <div className="flex items-center gap-2 font-bold text-blue-600">
                  <Shield className="w-4 h-4" /> PII_SCRUB_ON
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Cache</div>
                <div className="flex items-center gap-2 font-bold text-purple-600">
                  <RefreshCw className="w-4 h-4" /> SEMANTIC_Ready
                </div>
              </CardContent>
            </Card>
          </div>

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
                      {msg.suggestions && (
                        <div className="mt-2 flex flex-wrap gap-2">
                          {msg.suggestions.map((s: string, j: number) => (
                            <Badge key={j} onClick={() => setPrompt(s)} className="cursor-pointer border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-none">{s}</Badge>
                          ))}
                        </div>
                      )}
                      {msg.explanation && (
                        <div className="mt-4 p-4 bg-zinc-900 border-2 border-dashed border-zinc-700 text-zinc-400 text-sm flex justify-between items-end">
                          <div>-- {msg.explanation}</div>
                          {msg.role === 'ai' && (
                            <Button 
                              onClick={() => handleExport(history[i-1].content, msg.content)}
                              variant="outline" 
                              size="sm" 
                              className="h-8 border-zinc-700 text-xs font-black uppercase hover:bg-zinc-800 text-zinc-400"
                            >
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
                {isQuerying ? <Loader2 className="w-6 h-6 animate-spin" /> : <Send className="w-6 h-6" />}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
