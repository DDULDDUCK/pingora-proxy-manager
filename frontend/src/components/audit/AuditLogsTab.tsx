import { useState } from "react";
import { FileText, Filter, RefreshCw, Loader2, ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAuditLogs } from "@/hooks/useAuditLogs";
import type { AuditLog, AuditLogQuery } from "@/hooks/useAuditLogs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

const actionColors: Record<string, "default" | "secondary" | "destructive" | "outline"> = {
  login: "default",
  create: "default",
  update: "secondary",
  delete: "destructive",
  change_password: "outline",
  request: "default",
  upload: "default",
  add_client: "secondary",
  remove_client: "destructive",
  add_ip: "secondary",
  remove_ip: "destructive",
};

const actionLabels: Record<string, string> = {
  login: "Login",
  create: "Create",
  update: "Update",
  delete: "Delete",
  change_password: "Change Password",
  request: "Request",
  upload: "Upload",
  add_client: "Add Client",
  remove_client: "Remove Client",
  add_ip: "Add IP",
  remove_ip: "Remove IP",
};

const resourceTypeLabels: Record<string, string> = {
  session: "Session",
  user: "User",
  host: "Host",
  stream: "Stream",
  access_list: "Access List",
  certificate: "Certificate",
  location: "Location",
  settings: "Settings",
};

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function AuditLogsTab() {
  const { t } = useTranslation();
  const [query, setQuery] = useState<AuditLogQuery>({ limit: 100 });
  const [usernameFilter, setUsernameFilter] = useState("");
  const [resourceFilter, setResourceFilter] = useState("");
  const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null);
  const [detailOpen, setDetailOpen] = useState(false);

  const { data: logs, isLoading, refetch, isFetching } = useAuditLogs(query);

  const openDetail = (log: AuditLog) => {
    setSelectedLog(log);
    setDetailOpen(true);
  };

  const applyFilters = () => {
    const newQuery: AuditLogQuery = { limit: 100 };
    if (usernameFilter) newQuery.username = usernameFilter;
    if (resourceFilter && resourceFilter !== "all") newQuery.resource_type = resourceFilter;
    setQuery(newQuery);
  };

  const clearFilters = () => {
    setUsernameFilter("");
    setResourceFilter("");
    setQuery({ limit: 100 });
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <CardTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5" />
              {t('audit.title')}
            </CardTitle>
            <CardDescription>
              {t('audit.description')}
            </CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isFetching}
          >
            <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>

        {/* Filters */}
        <div className="flex flex-col md:flex-row gap-4 pt-4">
          <div className="flex-1 space-y-1">
            <Label htmlFor="username-filter">{t('audit.user')}</Label>
            <Input
              id="username-filter"
              value={usernameFilter}
              onChange={(e) => setUsernameFilter(e.target.value)}
              placeholder="Filter by username"
            />
          </div>
          <div className="flex-1 space-y-1">
            <Label htmlFor="resource-filter">Resource Type</Label>
            <Select value={resourceFilter} onValueChange={setResourceFilter}>
              <SelectTrigger>
                <SelectValue placeholder="All resources" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All resources</SelectItem>
                <SelectItem value="session">Session</SelectItem>
                <SelectItem value="user">User</SelectItem>
                <SelectItem value="host">Host</SelectItem>
                <SelectItem value="stream">Stream</SelectItem>
                <SelectItem value="access_list">Access List</SelectItem>
                <SelectItem value="cert">Certificate</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-end gap-2">
            <Button onClick={applyFilters}>
              <Filter className="mr-2 h-4 w-4" />
              Apply
            </Button>
            <Button variant="outline" onClick={clearFilters}>
              Clear
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[180px]">{t('audit.timestamp')}</TableHead>
                  <TableHead>{t('audit.user')}</TableHead>
                  <TableHead>{t('audit.action')}</TableHead>
                  <TableHead>Resource</TableHead>
                  <TableHead>{t('audit.details')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {logs?.map((log) => (
                  <TableRow
                    key={log.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => openDetail(log)}
                  >
                    <TableCell className="font-mono text-xs text-muted-foreground">
                      {formatTimestamp(log.timestamp)}
                    </TableCell>
                    <TableCell className="font-medium">{log.username}</TableCell>
                    <TableCell>
                      <Badge variant={actionColors[log.action] || "default"}>
                        {actionLabels[log.action] || log.action}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-col">
                        <span className="text-sm">
                          {resourceTypeLabels[log.resource_type] || log.resource_type}
                        </span>
                        {log.resource_id && (
                          <span className="text-xs text-muted-foreground">
                            ID: {log.resource_id}
                          </span>
                        )}
                      </div>
                    </TableCell>
                    <TableCell className="max-w-[300px]">
                      <div className="flex items-center gap-2">
                        <span className="text-sm text-muted-foreground truncate flex-1">
                          {log.details || "-"}
                        </span>
                        <ExternalLink className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
                {(!logs || logs.length === 0) && (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      No audit logs found
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </div>
        )}

        {logs && logs.length > 0 && (
          <div className="flex items-center justify-between pt-4">
            <p className="text-sm text-muted-foreground">
              Showing {logs.length} entries
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setQuery((q) => ({ ...q, limit: (q.limit || 100) + 100 }))}
            >
              Load More
            </Button>
          </div>
        )}
  
        {/* Detail Dialog */}
        <Dialog open={detailOpen} onOpenChange={setDetailOpen}>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <FileText className="h-5 w-5" />
                Audit Log Details
              </DialogTitle>
              <DialogDescription>
                {selectedLog && formatTimestamp(selectedLog.timestamp)}
              </DialogDescription>
            </DialogHeader>
            
            {selectedLog && (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1">
                    <Label className="text-muted-foreground">User</Label>
                    <p className="font-medium">{selectedLog.username}</p>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-muted-foreground">Action</Label>
                    <div>
                      <Badge variant={actionColors[selectedLog.action] || "default"}>
                        {actionLabels[selectedLog.action] || selectedLog.action}
                      </Badge>
                    </div>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-muted-foreground">Resource Type</Label>
                    <p className="font-medium">
                      {resourceTypeLabels[selectedLog.resource_type] || selectedLog.resource_type}
                    </p>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-muted-foreground">Resource ID</Label>
                    <p className="font-medium font-mono">
                      {selectedLog.resource_id || "-"}
                    </p>
                  </div>
                </div>
                
                <div className="space-y-1">
                  <Label className="text-muted-foreground">Details</Label>
                  <div className="bg-muted p-3 rounded-md">
                    <pre className="text-sm whitespace-pre-wrap break-all font-mono">
                      {selectedLog.details || "No details available"}
                    </pre>
                  </div>
                </div>
  
                {selectedLog.ip_address && (
                  <div className="space-y-1">
                    <Label className="text-muted-foreground">IP Address</Label>
                    <p className="font-mono text-sm">{selectedLog.ip_address}</p>
                  </div>
                )}
                
                <div className="space-y-1">
                  <Label className="text-muted-foreground">Log ID</Label>
                  <p className="font-mono text-sm text-muted-foreground">#{selectedLog.id}</p>
                </div>
              </div>
            )}
          </DialogContent>
        </Dialog>
      </CardContent>
    </Card>
    );
  }