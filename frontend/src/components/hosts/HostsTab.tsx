import { useState, useMemo } from "react";
import { RefreshCw } from "lucide-react";
import { useHosts, useDeleteHost, useIssueCert } from "@/hooks/useHosts";
import { useAuth } from "@/App";
import type { Host } from "@/hooks/useHosts";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card, CardContent, CardHeader, CardTitle, CardDescription,
} from "@/components/ui/card";
import { HostsTable } from "./HostsTable";
import { AddHostDialog } from "./AddHostDialog";
import { EditHostDialog } from "./EditHostDialog";

export function HostsTab() {
  const { canManageHosts } = useAuth();
  const { data: hosts, isLoading, refetch } = useHosts();
  const deleteHostMutation = useDeleteHost();
  const issueCertMutation = useIssueCert();

  const [searchQuery, setSearchQuery] = useState("");
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<Host | null>(null);

  const handleDeleteHost = (domain: string) => {
    if (!confirm(`Delete ${domain}?`)) return;
    deleteHostMutation.mutate(domain);
  };

  const handleIssueCert = (domain: string) => {
    issueCertMutation.mutate({ domain, email: "admin@example.com" });
  };

  const openEdit = (host: Host) => {
    setEditingHost(host);
    setIsEditOpen(true);
  };

  const filteredHosts = useMemo(() => (hosts || []).filter(h => 
    h.domain.toLowerCase().includes(searchQuery.toLowerCase())
  ), [hosts, searchQuery]);

  return (
    <Card>
       <CardHeader className="pb-3">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <CardTitle>Proxy Hosts</CardTitle>
                <CardDescription>Manage your routing and redirection rules.</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  placeholder="Search domains..."
                  className="w-[200px]"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
                <Button variant="outline" size="icon" onClick={() => refetch()} disabled={isLoading}>
                  <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                </Button>
                {canManageHosts && (
                  <AddHostDialog open={isAddOpen} onOpenChange={setIsAddOpen} />
                )}
              </div>
            </div>
       </CardHeader>
       <CardContent>
          <HostsTable 
            hosts={filteredHosts} 
            canManageHosts={canManageHosts} 
            onEdit={openEdit} 
            onIssueCert={handleIssueCert} 
            onDelete={handleDeleteHost} 
          />
       </CardContent>

       <EditHostDialog 
         host={editingHost} 
         open={isEditOpen} 
         onOpenChange={setIsEditOpen} 
       />
    </Card>
  );
}
