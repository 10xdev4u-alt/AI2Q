"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Database, Table as TableIcon, ArrowRight, Hash, Key, Link as LinkIcon } from "lucide-react"

export default function SchemaPage() {
  const [schema] = useState({
    tables: {
      "users": {
        name: "users",
        columns: [
          { name: "id", data_type: "uuid", is_primary_key: true },
          { name: "email", data_type: "text", is_primary_key: false },
          { name: "created_at", data_type: "timestamp", is_primary_key: false },
        ],
        foreign_keys: []
      },
      "orders": {
        name: "orders",
        columns: [
          { name: "id", data_type: "integer", is_primary_key: true },
          { name: "user_id", data_type: "uuid", is_primary_key: false },
          { name: "amount", data_type: "numeric", is_primary_key: false },
        ],
        foreign_keys: [
          { column_name: "user_id", foreign_table: "users", foreign_column: "id" }
        ]
      }
    }
  })

  return (
    <div className="min-h-screen bg-background p-8 font-mono">
      <div className="max-w-7xl mx-auto space-y-12">
        <div className="border-b-8 border-foreground pb-8">
          <h1 className="text-6xl font-black uppercase tracking-tighter italic">Interactive Schema Map</h1>
          <p className="text-2xl font-bold mt-4 border-l-8 border-primary pl-4">Visualize and manage your AI-ingested data structures.</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {Object.values(schema.tables).map((table) => (
            <Card key={table.name} className="border-4 border-foreground rounded-none shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] hover:shadow-none hover:translate-x-1 hover:translate-y-1 transition-all">
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
                    <div key={col.name} className="flex items-center justify-between p-2 border-2 border-foreground/10 group hover:border-primary transition-colors">
                      <div className="flex items-center gap-2">
                        {col.is_primary_key ? <Key className="w-4 h-4 text-yellow-500" /> : <Hash className="w-4 h-4" />}
                        <span className="font-bold">{col.name}</span>
                      </div>
                      <Badge variant="outline" className="rounded-none border-2 border-foreground">{col.data_type}</Badge>
                    </div>
                  ))}
                </div>

                {table.foreign_keys.length > 0 && (
                  <div className="space-y-2 pt-4 border-t-2 border-dashed border-foreground/20">
                    <div className="text-xs font-black text-muted-foreground uppercase mb-2">Relationships</div>
                    {table.foreign_keys.map((fk, i) => (
                      <div key={i} className="flex items-center gap-2 text-sm font-bold bg-primary/10 p-2 border-2 border-primary/20">
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
