"use client"

import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent } from "@/components/ui/card"
import { Terminal, Send, Zap, Shield, RefreshCw, Loader2, Braces, History } from "lucide-react"
import { aiqlApi, TranslateResult, Schema } from "@/lib/aiql"
import { Badge } from "@/components/ui/badge"

const DEFAULT_SCHEMA: Schema = {
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

export default function PlaygroundPage() {
  const [prompt, setPrompt] = useState("")
  const [dialect, setDialect] = useState("postgres")
  const [shouldExecute, setShouldExecute] = useState(false)
  const [dbUrl, setDbUrl] = useState("")
  const [schemaJson, setSchemaJson] = useState(JSON.stringify(DEFAULT_SCHEMA, null, 2))
  const [history, setHistory] = useState<any[]>([
    { role: "system", content: "AIQL Engine v1.0.0 Online. Standing by for natural language instructions." }
  ])
  const [isQuerying, setIsQuerying] = useState(false)
  const [recentQueries, setRecentQueries] = useState<string[]>([])

  useEffect(() => {
    const saved = localStorage.getItem("aiql_recent_queries")
    if (saved) setRecentQueries(JSON.parse(saved))
  }, [])

  const handleQuery = async () => {
    if (!prompt) return
    setIsQuerying(true)
    
    const updated = [prompt, ...recentQueries.filter(q => q !== prompt)].slice(0, 5)
    setRecentQueries(updated)
    localStorage.setItem("aiql_recent_queries", JSON.stringify(updated))

    setHistory(prev => [...prev, { role: "user", content: prompt }])
    
    try {
      let activeSchema: Schema;
      try {
        activeSchema = JSON.parse(schemaJson);
      } catch {
        activeSchema = DEFAULT_SCHEMA;
      }

      if (shouldExecute) {
        if (!dbUrl) throw new Error("Database URL is required for real execution")
        const result = await aiqlApi.ask(prompt, dbUrl, activeSchema)
        if (result.Success) {
          setHistory(prev => [...prev, { 
            role: "ai", 
            content: `Execution Successful!\nTime: ${result.Success.execution_time_ms}ms\nData: ${JSON.stringify(result.Success.data, null, 2)}`
          }])
        } else if (result.Error) {
          setHistory(prev => [...prev, { role: "system", content: `Execution Error: ${result.Error}` }])
        } else if (result.ClarificationNeeded) {
          setHistory(prev => [...prev, { 
            role: "system", 
            content: `Clarification Needed: ${result.ClarificationNeeded.reason}`,
            suggestions: result.ClarificationNeeded.suggestions
          }])
        }
      } else {
        const result = await aiqlApi.translate(prompt, activeSchema)
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
            <Card className="border-4 border-foreground rounded-none shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] bg-primary/10 overflow-hidden">
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

                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Execution Mode</div>
                <div className="flex items-center justify-between">
                  <span className="text-[10px] font-black uppercase">Real Execute</span>
                  <input 
                    type="checkbox" 
                    checked={shouldExecute} 
                    onChange={(e) => setShouldExecute(e.target.checked)}
                    className="w-4 h-4 border-2 border-foreground rounded-none accent-primary"
                  />
                </div>
                {shouldExecute && (
                  <Input 
                    value={dbUrl}
                    onChange={(e) => setDbUrl(e.target.value)}
                    placeholder="DB URL..."
                    className="h-8 border-2 border-foreground rounded-none text-[10px] font-bold bg-background text-foreground"
                  />
                )}

                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground flex items-center gap-2">
                  <Braces className="w-3 h-3" /> Schema Context
                </div>
                <textarea 
                  value={schemaJson}
                  onChange={(e) => setSchemaJson(e.target.value)}
                  className="w-full h-32 bg-background border-2 border-foreground p-2 font-mono text-[8px] focus:ring-0 outline-none text-foreground leading-tight"
                />

                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground flex items-center gap-2">
                  <History className="w-3 h-3" /> Recent Queries
                </div>
                <div className="space-y-1">
                  {recentQueries.map((q, i) => (
                    <div key={i} onClick={() => setPrompt(q)} className="text-[8px] font-bold border border-foreground/10 p-1 hover:bg-primary/20 cursor-pointer truncate uppercase text-foreground">
                      {q}
                    </div>
                  ))}
                  {recentQueries.length === 0 && <div className="text-[8px] italic opacity-50 uppercase text-foreground">No recent queries</div>}
                </div>

                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Engine Status</div>
                <div className="flex items-center gap-2 font-bold text-green-600 animate-pulse text-xs">
                  <Zap className="w-4 h-4" /> ACTIVE
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Security</div>
                <div className="flex items-center gap-2 font-bold text-blue-600 text-xs">
                  <Shield className="w-4 h-4" /> PII_SCRUB_ON
                </div>
                <div className="text-xs font-black uppercase border-b-2 border-foreground/20 pb-2 pt-2 text-foreground">Cache</div>
                <div className="flex items-center gap-2 font-bold text-purple-600 text-xs">
                  <RefreshCw className="w-4 h-4" /> SEMANTIC_Ready
                </div>
              </CardContent>
            </Card>
          </div>

          <div className="lg:col-span-3 flex flex-col h-[75vh] border-8 border-foreground shadow-[12px_12px_0px_0px_rgba(0,0,0,1)] bg-zinc-950 overflow-hidden text-foreground">
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
                            <Badge key={j} onClick={() => setPrompt(s)} className="cursor-pointer border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-none uppercase text-[10px]">{s}</Badge>
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
