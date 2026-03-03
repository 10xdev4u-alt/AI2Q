"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Database, Table as TableIcon, ArrowRight, Hash, Key, Link as LinkIcon, RefreshCw, Loader2 } from "lucide-react"
import { aiqlApi, Schema } from "@/lib/aiql"

export default function SchemaPage() {
  const [dbUrl, setDbUrl] = useState("")
  const [schema, setSchema] = useState<Schema | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleCrawl = async () => {
    if (!dbUrl) return
    setIsLoading(true)
    setError(null)
    try {
      const data = await aiqlApi.crawl(dbUrl)
      setSchema(data)
    } catch (err: any) {
      setError(err.response?.data?.error || err.message || "Failed to crawl schema")
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-background p-8 font-mono text-foreground">
      <div className="max-w-7xl mx-auto space-y-12">
        <div className="border-b-8 border-foreground pb-8 flex flex-col md:flex-row justify-between items-start md:items-end gap-6">
          <div>
            <h1 className="text-6xl font-black uppercase tracking-tighter italic leading-none text-foreground">Interactive Schema Map</h1>
            <p className="text-2xl font-bold mt-4 border-l-8 border-primary pl-4">Visualize and manage your AI-ingested data structures.</p>
          </div>
          <div className="flex gap-4 w-full md:w-auto">
            <Input 
              value={dbUrl}
              onChange={(e) => setDbUrl(e.target.value)}
              placeholder="postgresql://user:pass@host:5432/db"
              className="flex-1 md:w-96 border-4 border-foreground rounded-none h-12 font-bold bg-background text-foreground focus-visible:ring-0 focus-visible:border-primary"
            />
            <Button onClick={handleCrawl} disabled={isLoading} className="h-12 px-8 rounded-none border-4 border-foreground shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-1 hover:translate-y-1 transition-all">
              {isLoading ? <Loader2 className="w-6 h-6 animate-spin" /> : <RefreshCw className="w-6 h-6" />}
              <span className="ml-2 font-black">CRAWL</span>
            </Button>
          </div>
        </div>

        {error && (
          <div className="bg-red-500 text-white p-4 border-4 border-foreground font-black uppercase shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
            Error: {error}
          </div>
        )}

        {!schema && !isLoading && (
          <div className="py-20 text-center border-4 border-dashed border-foreground/20">
            <Database className="w-20 h-20 mx-auto opacity-20 mb-4" />
            <p className="text-2xl font-black opacity-20 uppercase tracking-widest">Connect a database to view its schema</p>
          </div>
        )}

        {isLoading && (
          <div className="py-20 text-center">
            <Loader2 className="w-20 h-20 mx-auto animate-spin text-primary" />
            <p className="text-2xl font-black mt-4 uppercase animate-pulse">Analyzing Database Topology...</p>
          </div>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 pb-20">
          {schema && Object.values(schema.tables).map((table) => (
            <Card key={table.name} className="border-4 border-foreground rounded-none shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-1 hover:translate-y-1 transition-all bg-card">
              <CardHeader className="bg-foreground text-background">
                <CardTitle className="flex items-center gap-3 text-2xl font-black uppercase">
                  <TableIcon className="w-8 h-8" />
                  {table.name}
                </CardTitle>
              </CardHeader>
              <CardContent className="p-6 space-y-4">
                <div className="space-y-2">
                  <div className="text-xs font-black text-muted-foreground uppercase mb-2">Columns</div>
                  {table.columns.map((col) => (
                    <div key={col.name} className="flex items-center justify-between p-2 border-2 border-foreground/10 group hover:border-primary transition-colors bg-background">
                      <div className="flex items-center gap-2">
                        {col.is_primary_key ? <Key className="w-4 h-4 text-yellow-500" /> : <Hash className="w-4 h-4" />}
                        <span className="font-bold">{col.name}</span>
                      </div>
                      <Badge variant="outline" className="rounded-none border-2 border-foreground text-foreground">{col.data_type}</Badge>
                    </div>
                  ))}
                </div>

                {table.foreign_keys.length > 0 && (
                  <div className="space-y-2 pt-4 border-t-2 border-dashed border-foreground/20">
                    <div className="text-xs font-black text-muted-foreground uppercase mb-2">Relationships</div>
                    {table.foreign_keys.map((fk, i) => (
                      <div key={i} className="flex items-center gap-2 text-sm font-bold bg-primary/10 p-2 border-2 border-primary/20 text-foreground">
                        <LinkIcon className="w-4 h-4" />
                        <span>{fk.column_name}</span>
                        <ArrowRight className="w-4 h-4" />
                        <span className="underline">{fk.foreign_table}</span>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}
