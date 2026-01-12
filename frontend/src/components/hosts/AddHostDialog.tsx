import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Plus, RefreshCw, Trash2, Server } from "lucide-react";
import type { Host } from "@/hooks/useHosts";
import { toast } from "sonner";
import { useAddHost } from "@/hooks/useHosts";
import { useAccessLists } from "@/hooks/useAccessLists";

interface AddHostDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AddHostDialog({ open, onOpenChange }: AddHostDialogProps) {
  const addHostMutation = useAddHost();
  const { data: accessLists } = useAccessLists();

  const [bulkMode, setBulkMode] = useState(false);
  const [bulkDomains, setBulkDomains] = useState("");
  const [newHost, setNewHost] = useState<Partial<Host>>({
    domain: "",
    target: "",
    scheme: "http",
    ssl_forced: false,
    redirect_to: "",
    redirect_status: 301,
    access_list_id: null
  });

  // Helper to get targets array from CSV string
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
        setTargets([""]); // Reset if last one
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
      
      if (domains.length === 0) return toast.warning("At least one domain is required");
      
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
              setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, redirect_to: "", redirect_status: 301, access_list_id: null });
              toast.success(`${created} hosts created successfully`);
            }
          }
        });
      });
      return;
    }
    
    if (!newHost.domain) return toast.warning("Domain is required");
    
    const hostPayload = {
        ...newHost,
        target: newHost.target || "127.0.0.1:8080",
        redirect_to: newHost.redirect_to || null,
        access_list_id: newHost.access_list_id === 0 ? null : newHost.access_list_id
    };

    addHostMutation.mutate(hostPayload, {
      onSuccess: () => {
        onOpenChange(false);
        setNewHost({ domain: "", target: "", scheme: "http", ssl_forced: false, redirect_to: "", redirect_status: 301, access_list_id: null });
      }
    });
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { onOpenChange(isOpen); if (!isOpen) setBulkMode(false); }}>
      <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" /> Add Host</Button></DialogTrigger>
      <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Add New Host</DialogTitle>
          <DialogDescription>Add one or multiple proxy hosts at once</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <div className="flex items-center justify-between">
              <Label>Domain{bulkMode ? "s" : ""}</Label>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setBulkMode(!bulkMode)}
                className="text-xs"
              >
                {bulkMode ? "Single Mode" : "Bulk Mode"}
              </Button>
            </div>
            {bulkMode ? (
              <Textarea
                value={bulkDomains}
                onChange={e => setBulkDomains(e.target.value)}
                placeholder="Enter multiple domains (comma or newline separated):\nexample1.com\nexample2.com\nexample3.com"
                rows={4}
              />
            ) : (
              <Input value={newHost.domain} onChange={e => setNewHost({...newHost, domain: e.target.value})} placeholder="example.com" />
            )}
            {bulkMode && (
              <p className="text-xs text-muted-foreground">
                All domains will share the same target, scheme, and other settings.
              </p>
            )}
          </div>
          
          <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                 <Label>Scheme</Label>
                 <Select value={newHost.scheme} onValueChange={v => setNewHost({...newHost, scheme: v as any})}>
                   <SelectTrigger><SelectValue /></SelectTrigger>
                   <SelectContent>
                     <SelectItem value="http">http://</SelectItem>
                     <SelectItem value="https">https://</SelectItem>
                   </SelectContent>
                 </Select>
              </div>
              
              {/* Multi-Target Input Section */}
              <div className="grid gap-2">
                <div className="flex items-center justify-between">
                    <Label>Target(s)</Label>
                    <Button type="button" variant="ghost" size="sm" onClick={handleAddTarget} className="h-6 px-2 text-xs">
                        <Plus className="h-3 w-3 mr-1" /> Add
                    </Button>
                </div>
                
                <div className="space-y-2 max-h-[150px] overflow-y-auto pr-1">
                    {getTargets().map((t, idx) => (
                        <div key={idx} className="flex gap-2">
                            <Input 
                                value={t} 
                                onChange={e => handleTargetChange(idx, e.target.value)} 
                                placeholder={idx === 0 ? "127.0.0.1:8080" : "10.0.0.2:8080"} 
                                className="flex-1"
                            />
                            {getTargets().length > 1 && (
                                <Button type="button" variant="ghost" size="icon" onClick={() => handleRemoveTarget(idx)}>
                                    <Trash2 className="h-4 w-4 text-red-500" />
                                </Button>
                            )}
                        </div>
                    ))}
                </div>
                <p className="text-[10px] text-muted-foreground">
                    Multiple targets enable <strong>Load Balancing</strong> (Random).
                </p>
              </div>
          </div>

          <div className="grid gap-2">
              <Label>Access List</Label>
              <Select value={newHost.access_list_id?.toString() || "0"} onValueChange={v => setNewHost({...newHost, access_list_id: parseInt(v)})}>
                  <SelectTrigger><SelectValue placeholder="Public (No Auth)" /></SelectTrigger>
                  <SelectContent>
                      <SelectItem value="0">Public (No Auth)</SelectItem>
                      {accessLists?.map(acl => (
                          <SelectItem key={acl.id} value={acl.id.toString()}>{acl.name}</SelectItem>
                      ))}
                  </SelectContent>
              </Select>
          </div>

          <div className="flex items-center space-x-2 pt-2">
              <input 
                type="checkbox" 
                id="ssl_forced" 
                className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                checked={newHost.ssl_forced || false}
                onChange={e => setNewHost({...newHost, ssl_forced: e.target.checked})}
              />
              <Label htmlFor="ssl_forced" className="cursor-pointer flex items-center">
                  Force SSL
                  <Badge variant="outline" className="ml-2 text-[10px] bg-yellow-50 text-yellow-700 border-yellow-200">HTTPS Only</Badge>
              </Label>
          </div>

          <div className="border-t pt-4 mt-2">
              <Label className="text-muted-foreground mb-2 block">Redirection (Optional)</Label>
              <div className="grid grid-cols-3 gap-4">
                  <div className="col-span-2 grid gap-2">
                      <Label className="text-xs">Redirect To URL</Label>
                      <Input value={newHost.redirect_to || ""} onChange={e => setNewHost({...newHost, redirect_to: e.target.value})} placeholder="https://google.com" />
                  </div>
                  <div className="grid gap-2">
                      <Label className="text-xs">Status Code</Label>
                      <Select value={newHost.redirect_status?.toString()} onValueChange={v => setNewHost({...newHost, redirect_status: parseInt(v)})}>
                        <SelectTrigger><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="301">301 (Perm)</SelectItem>
                          <SelectItem value="302">302 (Temp)</SelectItem>
                        </SelectContent>
                      </Select>
                  </div>
              </div>
          </div>
        </div>
        <DialogFooter>
           <Button onClick={handleAddHost} disabled={addHostMutation.isPending}>
               {addHostMutation.isPending && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
               Save
           </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
