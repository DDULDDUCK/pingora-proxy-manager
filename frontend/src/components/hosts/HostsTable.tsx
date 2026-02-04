import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { ArrowRightLeft, Link2, Lock, Settings, ShieldCheck, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Host } from "@/hooks/useHosts";
import type { Cert } from "@/hooks/useCertificates";

interface HostsTableProps {
  hosts: Host[];
  certs: Cert[];
  canManageHosts: boolean;
  onEdit: (host: Host) => void;
  onIssueCert: (domain: string) => void;
  onDelete: (domain: string) => void;
}

export function HostsTable({ hosts, certs, canManageHosts, onEdit, onIssueCert, onDelete }: HostsTableProps) {
  const { t } = useTranslation();
  const now = Math.floor(Date.now() / 1000);

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>{t('hosts.domain')}</TableHead>
          <TableHead>{t('hosts.destination')}</TableHead>
          <TableHead>{t('hosts.status')}</TableHead>
          <TableHead className="text-right">{t('hosts.actions')}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {hosts.map(host => {
          const cert = certs.find(c => c.domain === host.domain);
          let sslColor = "text-slate-400"; // Default: No cert
          let sslTitle = "No SSL Certificate";

          if (cert) {
            const daysLeft = Math.floor((cert.expires_at - now) / 86400);
            if (daysLeft < 0) {
              sslColor = "text-red-500";
              sslTitle = `Expired (${new Date(cert.expires_at * 1000).toLocaleDateString()})`;
            } else if (daysLeft < 30) {
              sslColor = "text-yellow-500";
              sslTitle = `Expires soon (${daysLeft} days)`;
            } else {
              sslColor = "text-green-500";
              sslTitle = `Valid until ${new Date(cert.expires_at * 1000).toLocaleDateString()}`;
            }
          }

          return (
            <TableRow key={host.domain}>
              <TableCell className="font-medium">
                <div className="flex flex-col">
                  <div className="flex items-center gap-2">
                    <span>{host.domain}</span>
                    {cert && <span title={sslTitle}><ShieldCheck className={`h-3 w-3 ${sslColor}`} /></span>}
                  </div>
                  <div className="flex gap-1 mt-1">
                    {host.ssl_forced && <Badge variant="outline" className="text-[10px] border-green-200 text-green-700 bg-green-50"><ShieldCheck className="h-3 w-3 mr-1" /> {t('hosts.httpsOnly')}</Badge>}
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
                      <Settings className="h-4 w-4 text-muted-foreground" />
                    </Button>
                  )}
                  {canManageHosts && (
                    <Button variant="ghost" size="sm" onClick={() => onIssueCert(host.domain)} title={sslTitle}>
                      <ShieldCheck className={`h-4 w-4 ${sslColor}`} />
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
          );
        })}
      </TableBody>
    </Table>
  );
}
