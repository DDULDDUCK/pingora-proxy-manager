import { useState, useMemo } from "react";
import { RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useHosts, useDeleteHost } from "@/hooks/useHosts";
import { useCertificates } from "@/hooks/useCertificates";
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
import { IssueCertDialog } from "./IssueCertDialog";

export function HostsTab() {
  const { t } = useTranslation();
  const { canManageHosts } = useAuth();
  const { data: hosts, isLoading, refetch } = useHosts();
  const { certs } = useCertificates();
  const deleteHostMutation = useDeleteHost();

  const [searchQuery, setSearchQuery] = useState("");
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [isCertOpen, setIsCertOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<Host | null>(null);
  const [certDomain, setCertDomain] = useState<string | null>(null);

  const handleDeleteHost = (domain: string) => {
    if (!confirm(t('hosts.deleteConfirm', { domain }))) return;
    deleteHostMutation.mutate(domain);
  };

  const handleIssueCert = (domain: string) => {
    setCertDomain(domain);
    setIsCertOpen(true);
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
                <CardTitle>{t('hosts.title')}</CardTitle>
                <CardDescription>{t('hosts.description')}</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  placeholder={t('hosts.searchDomains')}
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
            certs={certs}
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

       <IssueCertDialog
         domain={certDomain}
         open={isCertOpen}
         onOpenChange={setIsCertOpen}
       />
    </Card>
  );
}
