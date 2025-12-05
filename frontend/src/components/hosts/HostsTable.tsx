import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { ArrowRightLeft, Link2, Lock, Settings, ShieldCheck, Trash2 } from "lucide-react";
import type { Host } from "@/hooks/useHosts";

interface HostsTableProps {
  hosts: Host[];
  canManageHosts: boolean;
  onEdit: (host: Host) => void;
  onIssueCert: (domain: string) => void;
  onDelete: (domain: string) => void;
}

export function HostsTable({ hosts, canManageHosts, onEdit, onIssueCert, onDelete }: HostsTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Domain</TableHead>
          <TableHead>Destination</TableHead>
          <TableHead>Status</TableHead>
          <TableHead className="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {hosts.map(host => (
          <TableRow key={host.domain}>
            <TableCell className="font-medium">
              <div className="flex flex-col">
                <span>{host.domain}</span>
                <div className="flex gap-1 mt-1">
                  {host.ssl_forced && <Badge variant="outline" className="text-[10px] border-green-200 text-green-700 bg-green-50"><ShieldCheck className="h-3 w-3 mr-1" /> SSL</Badge>}
                  {host.access_list_id && <Badge variant="outline" className="text-[10px] border-orange-200 text-orange-700 bg-orange-50"><Lock className="h-3 w-3 mr-1" /> ACL</Badge>}
                </div>
              </div>
            </TableCell>
            <TableCell>
              {host.redirect_to ? (
                <div className="flex items-center text-blue-600">
                  <Link2 className="h-3 w-3 mr-1" />
                  <span className="text-xs font-mono">{host.redirect_to}</span>
                  <Badge variant="secondary" className="ml-2 text-[10px]">{host.redirect_status}</Badge>
                </div>
              ) : (
                <div className="flex items-center">
                  <span className="text-sm">{host.target}</span>
                  <Badge variant="outline" className="ml-2 text-[10px]">{host.scheme}</Badge>
                </div>
              )}
            </TableCell>
            <TableCell>
              {host.locations && host.locations.length > 0 ? (
                <div className="flex flex-wrap gap-1">
                  {host.locations.map(l => (
                    <Badge key={l.path} variant="secondary" className="text-[10px]">
                      {l.path} → {l.target}
                      {l.rewrite && <ArrowRightLeft className="ml-1 h-3 w-3 inline" />}
                    </Badge>
                  ))}
                </div>
              ) : (
                <span className="text-muted-foreground text-xs">Default only</span>
              )}
            </TableCell>
            <TableCell className="text-right">
              <div className="flex items-center justify-end gap-1">
                {canManageHosts && (
                  <Button variant="ghost" size="sm" onClick={() => onEdit(host)}>
                    <Settings className="h-4 w-4 text-slate-500" />
                  </Button>
                )}
                {canManageHosts && (
                  <Button variant="ghost" size="sm" onClick={() => onIssueCert(host.domain)}>
                    <ShieldCheck className="h-4 w-4 text-blue-500" />
                  </Button>
                )}
                {canManageHosts && (
                  <Button variant="ghost" size="sm" onClick={() => onDelete(host.domain)}>
                    <Trash2 className="h-4 w-4 text-red-500" />
                  </Button>
                )}
              </div>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
