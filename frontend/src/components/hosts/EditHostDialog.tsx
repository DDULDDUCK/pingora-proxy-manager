import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Edit, RefreshCw, CornerDownRight, ArrowRightLeft, Trash2, Plus, ShieldAlert, Globe, Server, Shield, Redo2 } from "lucide-react";
import type { Host, Location, Header } from "@/hooks/useHosts";
import { toast } from "sonner";
import { useAddHost, useAddLocation, useDeleteLocation, useAddHostHeader, useDeleteHostHeader } from "@/hooks/useHosts";
import { useAccessLists } from "@/hooks/useAccessLists";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface EditHostDialogProps {
  host: Host | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const Section = ({ title, icon: Icon, children, className }: { title: string, icon: any, children: React.ReactNode, className?: string }) => (
  <div className={cn("p-4 rounded-lg border bg-muted/30 space-y-3", className)}>
    <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground border-b pb-2 mb-2 uppercase tracking-wider">
      <Icon className="h-4 w-4" />
      {title}
    </div>
    {children}
  </div>
);

export function EditHostDialog({ host, open, onOpenChange }: EditHostDialogProps) {
  const { t } = useTranslation();
  const addHostMutation = useAddHost();
  const addLocationMutation = useAddLocation();
  const deleteLocationMutation = useDeleteLocation();
  const addHeaderMutation = useAddHostHeader();
  const deleteHeaderMutation = useDeleteHostHeader();
  const { data: accessLists } = useAccessLists();

  const [editFormHost, setEditFormHost] = useState<Partial<Host>>({});
  const [newLocation, setNewLocation] = useState<Location>({ path: "/", target: "", scheme: "http", rewrite: false, verify_ssl: true, upstream_sni: "" });
  const [newHeader, setNewHeader] = useState<Omit<Header, 'id'>>({ name: "", value: "", target: "request" });

  useEffect(() => {
    if (host) {
        setEditFormHost({
            target: host.target,
            scheme: host.scheme,
            ssl_forced: host.ssl_forced,
            verify_ssl: host.verify_ssl,
            upstream_sni: host.upstream_sni || "",
            redirect_to: host.redirect_to || "",
            redirect_status: host.redirect_status || 301,
            access_list_id: host.access_list_id || null,
        });
    }
  }, [host]);

  const getTargets = () => {
    if (!editFormHost.target) return [""];
    return editFormHost.target.split(',').map(t => t.trim());
  };

  const setTargets = (targets: string[]) => {
    setEditFormHost({ ...editFormHost, target: targets.join(',') });
  };

  const handleAddTarget = () => {
    const current = getTargets();
    setTargets([...current, ""]);
  };

  const handleRemoveTarget = (index: number) => {
    const current = getTargets();
    if (current.length <= 1) {
        setTargets([""]);
        return;
    }
    const next = current.filter((_, i) => i !== index);
    setTargets(next);
  };

  const handleTargetChange = (index: number, value: string) => {
    const current = getTargets();
    current[index] = value;
    setTargets(current);
  };

  const handleUpdateHost = () => {
    if (!host) return;
    
    const hostPayload = {
      ...editFormHost,
      domain: host.domain,
      redirect_to: editFormHost.redirect_to || null,
      access_list_id: editFormHost.access_list_id === 0 ? null : editFormHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        toast.success(t('hosts.updateSuccess'));
        onOpenChange(false);
      }
    });
  };

  const handleAddLocation = () => {
    if (!host || !newLocation.path || !newLocation.target) return;
    addLocationMutation.mutate({ domain: host.domain, location: newLocation }, {
        onSuccess: () => {
            setNewLocation({ path: "/", target: "", scheme: "http", rewrite: false, verify_ssl: true, upstream_sni: "" });
        }
    });
  };

  const handleDeleteLocation = (path: string) => {
    if (!host) return;
    deleteLocationMutation.mutate({ domain: host.domain, path });
  };

  const handleAddHeader = () => {
    if (!host || !newHeader.name || !newHeader.value) return toast.warning(t('hosts.headerWarning'));
    addHeaderMutation.mutate({ domain: host.domain, header: newHeader }, {
        onSuccess: () => {
            setNewHeader({ name: "", value: "", target: "request" });
            toast.success(t('hosts.headerAddSuccess'));
        },
        onError: (error) => {
            toast.error(t('hosts.headerAddError', { error: error.message }));
        }
    });
  };

  const handleDeleteHeader = (headerId: number) => {
    if (!host) return;
    if (!confirm(t('hosts.headerDeleteConfirm'))) return;
    deleteHeaderMutation.mutate({ domain: host.domain, headerId }, {
        onSuccess: () => {
            toast.success(t('hosts.headerDeleteSuccess'));
        },
        onError: (error) => {
            toast.error(t('hosts.headerDeleteError', { error: error.message }));
        }
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-xl">
            <Edit className="h-5 w-5 text-primary" />
            {t('hosts.editHost')}: <span className="text-primary font-mono">{host?.domain}</span>
          </DialogTitle>
          <DialogDescription>
            {t('hosts.editDescription')}
          </DialogDescription>
        </DialogHeader>
        
        <Tabs defaultValue="settings" className="w-full">
          <TabsList className="grid w-full grid-cols-3 mb-6">
            <TabsTrigger value="settings" className="flex items-center gap-2">
              <Server className="h-4 w-4" /> {t('hosts.hostSettings')}
            </TabsTrigger>
            <TabsTrigger value="locations" className="flex items-center gap-2">
              <CornerDownRight className="h-4 w-4" /> {t('hosts.locations')} ({host?.locations?.length || 0})
            </TabsTrigger>
            <TabsTrigger value="headers" className="flex items-center gap-2">
              <Plus className="h-4 w-4" /> {t('hosts.customHeaders')} ({host?.headers?.length || 0})
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="settings" className="space-y-6">
            <div className="flex flex-col gap-6">
              <Section title={t('hosts.general')} icon={Globe}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 items-end">
                  <div className="grid gap-2">
                    <Label className="text-muted-foreground">{t('hosts.domain')}</Label>
                    <div className="px-3 py-2 bg-muted rounded-md font-mono text-sm border">
                      {host?.domain}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2 pb-3">
                    <input 
                      type="checkbox"
                      id="edit_ssl_forced" 
                      className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                      checked={editFormHost.ssl_forced || false}
                      onChange={(e) => setEditFormHost({...editFormHost, ssl_forced: e.target.checked})}
                    />
                    <Label htmlFor="edit_ssl_forced" className="cursor-pointer text-sm font-medium leading-none">
                      {t('hosts.forceSsl')}
                    </Label>
                    <Badge variant="outline" className="text-[10px] bg-yellow-50 text-yellow-700 border-yellow-200">HTTPS Only</Badge>
                  </div>
                </div>
              </Section>

              <Section title={t('hosts.upstream')} icon={Server}>
                <div className="grid grid-cols-1 md:grid-cols-12 gap-8">
                  <div className="md:col-span-5 space-y-4">
                    <div className="grid gap-2">
                      <Label>{t('hosts.scheme')}</Label>
                      <Select value={editFormHost.scheme} onValueChange={v => setEditFormHost({...editFormHost, scheme: v as any})}>
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="http">http://</SelectItem>
                          <SelectItem value="https">https://</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="grid gap-2">
                      <div className="flex items-center gap-2">
                        <Label htmlFor="edit_upstream_sni" className="text-xs whitespace-nowrap">{t('hosts.upstreamSni')}</Label>
                      </div>
                      <Input 
                        id="edit_upstream_sni"
                        value={editFormHost.upstream_sni || ""} 
                        onChange={e => setEditFormHost({...editFormHost, upstream_sni: e.target.value})} 
                        placeholder={getTargets()[0]?.split(':')[0] || t('hosts.upstreamSniPlaceholder')}
                        className="text-sm"
                      />
                      <p className="text-[10px] text-muted-foreground leading-tight italic">
                        {t('hosts.upstreamSniHelp')} <span className="text-blue-600 font-medium not-italic">({t('hosts.upstreamSniBadgeEdit')})</span>
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <input 
                        type="checkbox"
                        id="edit_verify_ssl" 
                        className="h-3 w-3 rounded border-gray-300 text-primary focus:ring-primary"
                        checked={editFormHost.verify_ssl ?? true}
                        onChange={(e) => setEditFormHost({...editFormHost, verify_ssl: e.target.checked})}
                      />
                      <Label htmlFor="edit_verify_ssl" className="cursor-pointer text-xs text-muted-foreground">
                        {t('hosts.verifySsl')}
                      </Label>
                    </div>
                  </div>
                  
                  <div className="md:col-span-7 grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>{t('hosts.target')}</Label>
                      <Button type="button" variant="outline" size="sm" onClick={handleAddTarget} className="h-7 px-2 text-[10px] font-bold uppercase tracking-wider">
                        <Plus className="h-3 w-3 mr-1" /> {t('common.add')}
                      </Button>
                    </div>
                    <div className="space-y-2 max-h-[160px] overflow-y-auto pr-1">
                      {getTargets().map((t_val, idx) => (
                        <div key={idx} className="flex gap-2">
                          <Input 
                            value={t_val} 
                            onChange={e => handleTargetChange(idx, e.target.value)} 
                            placeholder={idx === 0 ? "127.0.0.1:8080" : "10.0.0.2:8080"} 
                            className="flex-1 text-sm font-mono"
                          />
                          {getTargets().length > 1 && (
                            <Button type="button" variant="ghost" size="icon" onClick={() => handleRemoveTarget(idx)} className="h-9 w-9">
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          )}
                        </div>
                      ))}
                    </div>
                    <p className="text-[10px] text-muted-foreground italic">
                      {t('hosts.loadBalancingHelp')}
                    </p>
                  </div>
                </div>
              </Section>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Section title={t('hosts.accessControl')} icon={Shield} className="h-full">
                  <div className="grid gap-2">
                    <Label>{t('hosts.accessList')}</Label>
                    <Select value={editFormHost.access_list_id?.toString() || "0"} onValueChange={v => setEditFormHost({...editFormHost, access_list_id: parseInt(v)})}>
                      <SelectTrigger><SelectValue placeholder={t('hosts.publicNoAuth')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="0">{t('hosts.publicNoAuth')}</SelectItem>
                        {accessLists?.map(acl => (
                          <SelectItem key={acl.id} value={acl.id.toString()}>{acl.name}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </Section>

                <Section title={t('hosts.redirection')} icon={Redo2} className="h-full">
                  <div className="grid gap-4">
                    <div className="grid gap-2">
                      <Label className="text-xs">{t('hosts.redirectToUrl')}</Label>
                      <Input 
                        value={editFormHost.redirect_to || ""} 
                        onChange={e => setEditFormHost({...editFormHost, redirect_to: e.target.value})} 
                        placeholder="https://google.com" 
                        className="text-sm"
                      />
                    </div>
                    <div className="grid gap-2">
                      <Label className="text-xs">{t('hosts.statusCode')}</Label>
                      <Select value={editFormHost.redirect_status?.toString()} onValueChange={v => setEditFormHost({...editFormHost, redirect_status: parseInt(v)})}>
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="301">301 (Permanent)</SelectItem>
                          <SelectItem value="302">302 (Temporary)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </Section>
              </div>
            </div>


            <DialogFooter className="border-t pt-6 mt-4">
              <Button onClick={handleUpdateHost} disabled={addHostMutation.isPending} className="w-full md:w-auto px-8">
                {addHostMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                {t('common.saveChanges')}
              </Button>
            </DialogFooter>
          </TabsContent>
          
          <TabsContent value="locations" className="space-y-6 pt-2">
            <div className="p-4 rounded-lg border bg-muted/30">
              <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground mb-4">
                <Plus className="h-4 w-4" /> {t('hosts.addPathRouting')}
              </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                <div className="grid gap-2">
                  <Label className="text-xs">{t('hosts.pathPrefix')}</Label>
                  <Input value={newLocation.path} onChange={e => setNewLocation({...newLocation, path: e.target.value})} placeholder="/api" className="text-sm" />
                </div>
                <div className="grid gap-2">
                  <Label className="text-xs">{t('hosts.target')}</Label>
                  <Input value={newLocation.target} onChange={e => setNewLocation({...newLocation, target: e.target.value})} placeholder="10.0.0.5:3000, 10.0.0.6:3000" className="text-sm font-mono" />
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 items-end">
                <div className="grid gap-2">
                  <Label className="text-xs">{t('hosts.scheme')}</Label>
                  <Select value={newLocation.scheme} onValueChange={v => setNewLocation({...newLocation, scheme: v as any})}>
                    <SelectTrigger className="h-9"><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="http">http</SelectItem>
                      <SelectItem value="https">https</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                 <div className="flex flex-col gap-2">
                    <div className="flex items-center space-x-2">
                       <input 
                         type="checkbox"
                         id="loc_verify_ssl" 
                         className="h-3 w-3 rounded border-gray-300 text-primary focus:ring-primary"
                         checked={newLocation.verify_ssl ?? true}
                         onChange={(e) => setNewLocation({...newLocation, verify_ssl: e.target.checked})}
                       />
                       <Label htmlFor="loc_verify_ssl" className="cursor-pointer text-[10px] text-muted-foreground whitespace-nowrap">
                           {t('hosts.verifySsl')}
                       </Label>
                    </div>
                    <div className="flex items-center space-x-2">
                       <input
                         type="checkbox"
                         id="rewrite"
                         className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                         checked={newLocation.rewrite || false}
                         onChange={(e) => setNewLocation({...newLocation, rewrite: e.target.checked})}
                       />
                       <Label htmlFor="rewrite" className="cursor-pointer text-[10px] text-muted-foreground">{t('hosts.stripPath')}</Label>
                     </div>
                 </div>
                <Button className="h-9" onClick={handleAddLocation} disabled={addLocationMutation.isPending}>
                  <Plus className="h-4 w-4 mr-2" /> {t('common.add')}
                </Button>
              </div>
              <div className="mt-4 grid gap-2">
                  <Label htmlFor="loc_upstream_sni" className="text-[10px] text-muted-foreground">{t('hosts.upstreamSni')}</Label>
                  <Input 
                    id="loc_upstream_sni"
                    value={newLocation.upstream_sni || ""} 
                    onChange={e => setNewLocation({...newLocation, upstream_sni: e.target.value})} 
                    placeholder={newLocation.target?.split(',')[0]?.split(':')[0] || t('hosts.upstreamSniPlaceholder')}
                    className="h-8 text-[10px]"
                  />
              </div>
            </div>

            <div className="space-y-3">
              <Label className="text-base">{t('hosts.configuredLocations')}</Label>
              {host?.locations?.length === 0 && (
                <div className="text-sm text-muted-foreground italic p-8 text-center border rounded-lg bg-muted/10">
                  {t('hosts.noLocations')}
                </div>
              )}
              {host?.locations?.map((loc) => (
                <div key={loc.path} className="flex items-center justify-between p-3 bg-card rounded-lg border shadow-sm transition-all hover:border-primary/30">
                  <div className="flex items-center gap-3 flex-wrap">
                    <CornerDownRight className="h-4 w-4 text-muted-foreground" />
                    <Badge variant="secondary" className="font-mono">{loc.path}</Badge>
                    <ArrowRightLeft className="h-3 w-3 text-muted-foreground" />
                    <span className="font-mono text-sm font-semibold">{loc.scheme}://{loc.target}</span>
                    {loc.upstream_sni && (
                      <Badge variant="outline" className="text-[10px] bg-blue-50 text-blue-700 border-blue-200">
                        SNI: {loc.upstream_sni}
                      </Badge>
                    )}
                    {loc.verify_ssl === false && (
                      <Badge variant="destructive" className="text-[10px] h-5 flex items-center gap-1">
                        <ShieldAlert className="h-3 w-3" /> {t('hosts.noSslVerify')}
                      </Badge>
                    )}
                    {loc.rewrite && (
                      <Badge variant="secondary" className="text-[10px] h-5 bg-orange-50 text-orange-700 border-orange-200">
                        {t('hosts.stripped')}
                      </Badge>
                    )}
                  </div>
                  <Button variant="ghost" size="icon" onClick={() => handleDeleteLocation(loc.path)} disabled={deleteLocationMutation.isPending} className="h-8 w-8 text-destructive hover:bg-destructive/10">
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="headers" className="space-y-6 pt-2">
            <div className="p-4 rounded-lg border bg-muted/30">
                <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground mb-4">
                  <Plus className="h-4 w-4" /> {t('hosts.addNewHeader')}
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                    <div className="grid gap-2">
                        <Label className="text-xs">{t('hosts.headerName')}</Label>
                        <Input 
                            value={newHeader.name} 
                            onChange={e => setNewHeader({...newHeader, name: e.target.value})} 
                            placeholder="X-Custom-Header" 
                            className="text-sm font-mono"
                        />
                    </div>
                    <div className="grid gap-2">
                        <Label className="text-xs">{t('hosts.headerValue')}</Label>
                        <Input 
                            value={newHeader.value} 
                            onChange={e => setNewHeader({...newHeader, value: e.target.value})} 
                            placeholder="my-value" 
                            className="text-sm font-mono"
                        />
                    </div>
                </div>
                <div className="flex items-end gap-4">
                    <div className="grid gap-2 w-[180px]">
                        <Label className="text-xs">{t('hosts.targetType')}</Label>
                        <Select 
                            value={newHeader.target} 
                            onValueChange={v => setNewHeader({...newHeader, target: v as "request" | "response"})}
                        >
                            <SelectTrigger className="h-9"><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem value="request">{t('hosts.requestHeader')}</SelectItem>
                                <SelectItem value="response">{t('hosts.responseHeader')}</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <Button className="ml-auto h-9" onClick={handleAddHeader} disabled={addHeaderMutation.isPending}>
                        <Plus className="h-4 w-4 mr-2" /> {t('hosts.addHeader')}
                    </Button>
                </div>
            </div>

            <div className="space-y-3">
                <Label className="text-base">{t('hosts.configuredHeaders')}</Label>
                {host?.headers?.length === 0 && (
                  <div className="text-sm text-muted-foreground italic p-8 text-center border rounded-lg bg-muted/10">
                    {t('hosts.noHeaders')}
                  </div>
                )}
                {host?.headers && host.headers.length > 0 && (
                  <div className="rounded-lg border shadow-sm overflow-hidden">
                    <Table>
                        <TableHeader className="bg-muted/50">
                            <TableRow>
                                <TableHead>{t('hosts.headerName')}</TableHead>
                                <TableHead>{t('hosts.headerValue')}</TableHead>
                                <TableHead>{t('hosts.targetType')}</TableHead>
                                <TableHead className="w-[80px] text-right">{t('common.actions')}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {host?.headers?.map((header) => (
                                <TableRow key={header.id} className="hover:bg-muted/30">
                                    <TableCell className="font-mono text-sm font-medium">{header.name}</TableCell>
                                    <TableCell className="font-mono text-sm">{header.value}</TableCell>
                                    <TableCell>
                                      <Badge variant={header.target === 'request' ? 'default' : 'outline'} className="capitalize text-[10px]">
                                        {header.target}
                                      </Badge>
                                    </TableCell>
                                    <TableCell className="text-right">
                                        <Button variant="ghost" size="icon" onClick={() => handleDeleteHeader(header.id)} disabled={deleteHeaderMutation.isPending} className="h-8 w-8 text-destructive hover:bg-destructive/10">
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                  </div>
                )}
            </div>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
