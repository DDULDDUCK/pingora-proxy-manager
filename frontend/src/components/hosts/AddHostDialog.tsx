import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Plus, RefreshCw, Trash2, Globe, Server, Shield, Redo2 } from "lucide-react";
import type { Host } from "@/hooks/useHosts";
import { toast } from "sonner";
import { useAddHost } from "@/hooks/useHosts";
import { useAccessLists } from "@/hooks/useAccessLists";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface AddHostDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const Section = ({ title, icon: Icon, children, className }: { title: string, icon: any, children: React.ReactNode, className?: string }) => (
  <div className={cn("p-4 rounded-lg border bg-muted/30 space-y-3", className)}>
    <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground border-b pb-2 mb-2">
      <Icon className="h-4 w-4" />
      {title}
    </div>
    {children}
  </div>
);

export function AddHostDialog({ open, onOpenChange }: AddHostDialogProps) {
  const { t } = useTranslation();
  const addHostMutation = useAddHost();
  const { data: accessLists } = useAccessLists();

  const [bulkMode, setBulkMode] = useState(false);
  const [bulkDomains, setBulkDomains] = useState("");
  const [newHost, setNewHost] = useState<Partial<Host>>({
    domain: "",
    target: "",
    scheme: "http",
    ssl_forced: false,
    verify_ssl: true,
    upstream_sni: "",
    redirect_to: "",
    redirect_status: 301,
    access_list_id: null
  });

  const getTargets = () => {
    if (!newHost.target) return [""];
    return newHost.target.split(',').map(t => t.trim());
  };

  const setTargets = (targets: string[]) => {
    setNewHost({ ...newHost, target: targets.join(',') });
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

  const handleAddHost = () => {
    if (bulkMode) {
      const domains = bulkDomains
        .split(/[,\n]/)
        .map(d => d.trim())
        .filter(d => d.length > 0);
      
      if (domains.length === 0) return toast.warning(t('hosts.bulkWarning'));
      
      let created = 0;
      domains.forEach((domain) => {
        const hostPayload = {
          ...newHost,
          domain,
          target: newHost.target || "127.0.0.1:8080",
          redirect_to: newHost.redirect_to || null,
          access_list_id: newHost.access_list_id === 0 ? null : newHost.access_list_id
        };
        
        addHostMutation.mutate(hostPayload, {
          onSuccess: () => {
            created++;
            if (created === domains.length) {
              onOpenChange(false);
              setBulkDomains("");
              setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, verify_ssl: true, upstream_sni: "", redirect_to: "", redirect_status: 301, access_list_id: null });
              toast.success(t('hosts.bulkSuccess', { count: created }));
            }
          }
        });
      });
      return;
    }
    
    if (!newHost.domain) return toast.warning(t('hosts.domainRequired'));
    
    const hostPayload = {
      ...newHost,
      target: newHost.target || "127.0.0.1:8080",
      redirect_to: newHost.redirect_to || null,
      access_list_id: newHost.access_list_id === 0 ? null : newHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        onOpenChange(false);
        setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, verify_ssl: true, upstream_sni: "", redirect_to: "", redirect_status: 301, access_list_id: null });
      }
    });
  };


  return (
    <Dialog open={open} onOpenChange={(isOpen) => { onOpenChange(isOpen); if (!isOpen) setBulkMode(false); }}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="mr-2 h-4 w-4" /> {t('hosts.addHost')}
        </Button>
      </DialogTrigger>
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('hosts.addNewHost')}</DialogTitle>
          <DialogDescription>{t('hosts.addDescription')}</DialogDescription>
        </DialogHeader>
        
        <div className="flex flex-col gap-6 py-4">
          <Section title={t('hosts.general')} icon={Globe}>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 items-start">
              <div className="grid gap-2">
                <div className="flex items-center justify-between">
                  <Label>{t('hosts.domain')}{bulkMode ? "s" : ""}</Label>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => setBulkMode(!bulkMode)}
                    className="text-[10px] h-6 px-2 uppercase tracking-wider font-bold"
                  >
                    {bulkMode ? t('hosts.singleMode') : t('hosts.bulkMode')}
                  </Button>
                </div>
                {bulkMode ? (
                  <Textarea
                    value={bulkDomains}
                    onChange={e => setBulkDomains(e.target.value)}
                    placeholder={t('hosts.bulkPlaceholder')}
                    rows={3}
                    className="text-sm"
                  />
                ) : (
                  <Input 
                    value={newHost.domain} 
                    onChange={e => setNewHost({...newHost, domain: e.target.value})} 
                    placeholder="example.com" 
                  />
                )}
                {bulkMode && (
                  <p className="text-[10px] text-muted-foreground leading-tight">
                    {t('hosts.bulkHelp')}
                  </p>
                )}
              </div>
              <div className="flex items-center space-x-2 self-end pb-3">
                <input 
                  type="checkbox"
                  id="ssl_forced" 
                  className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                  checked={newHost.ssl_forced || false}
                  onChange={(e) => setNewHost({...newHost, ssl_forced: e.target.checked})}
                />
                <Label htmlFor="ssl_forced" className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
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
                  <Select value={newHost.scheme} onValueChange={v => setNewHost({...newHost, scheme: v as any})}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="http">http://</SelectItem>
                      <SelectItem value="https">https://</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid gap-2">
                  <div className="flex items-center gap-2">
                    <Label htmlFor="upstream_sni" className="text-xs whitespace-nowrap">{t('hosts.upstreamSni')}</Label>
                  </div>
                  <Input 
                    id="upstream_sni"
                    value={newHost.upstream_sni || ""} 
                    onChange={e => setNewHost({...newHost, upstream_sni: e.target.value})} 
                    placeholder={getTargets()[0]?.split(':')[0] || t('hosts.upstreamSniPlaceholder')}
                    className="text-sm"
                  />
                  <p className="text-[10px] text-muted-foreground leading-tight italic">
                    {t('hosts.upstreamSniHelp')} <span className="text-blue-600 font-medium not-italic">({t('hosts.upstreamSniBadge')})</span>
                  </p>
                </div>
                <div className="flex items-center space-x-2">
                  <input 
                    type="checkbox" 
                    id="verify_ssl" 
                    className="h-3 w-3 rounded border-gray-300 text-primary focus:ring-primary"
                    checked={newHost.verify_ssl ?? true}
                    onChange={(e) => setNewHost({...newHost, verify_ssl: e.target.checked})}
                  />
                  <Label htmlFor="verify_ssl" className="cursor-pointer text-xs text-muted-foreground">
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
                <Select value={newHost.access_list_id?.toString() || "0"} onValueChange={v => setNewHost({...newHost, access_list_id: parseInt(v)})}>
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
                    value={newHost.redirect_to || ""} 
                    onChange={e => setNewHost({...newHost, redirect_to: e.target.value})} 
                    placeholder="https://google.com" 
                    className="text-sm"
                  />
                </div>
                <div className="grid gap-2">
                  <Label className="text-xs">{t('hosts.statusCode')}</Label>
                  <Select value={newHost.redirect_status?.toString()} onValueChange={v => setNewHost({...newHost, redirect_status: parseInt(v)})}>
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


        <DialogFooter className="border-t pt-4">
          <Button onClick={handleAddHost} disabled={addHostMutation.isPending} className="w-full md:w-auto px-8">
            {addHostMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
