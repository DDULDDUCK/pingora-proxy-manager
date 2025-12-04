import { useState, useMemo } from "react";
import { RefreshCw, Plus, Settings, ShieldCheck, Trash2, CornerDownRight, ArrowRightLeft } from "lucide-react";
import { useHosts, useAddHost, useDeleteHost, useAddLocation, useDeleteLocation, useIssueCert } from "@/hooks/useHosts";
import type { Host, Location } from "@/hooks/useHosts";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter,
} from "@/components/ui/dialog";
import {
  Card, CardContent, CardHeader, CardTitle, CardDescription,
} from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";

export function HostsTab() {
  const { data: hosts, isLoading, refetch } = useHosts();
  const addHostMutation = useAddHost();
  const deleteHostMutation = useDeleteHost();
  const addLocationMutation = useAddLocation();
  const deleteLocationMutation = useDeleteLocation();
  const issueCertMutation = useIssueCert();

  const [searchQuery, setSearchQuery] = useState("");
  
  // Add Host Dialog
  const [isAddOpen, setIsAddOpen] = useState(false);
  const [newHost, setNewHost] = useState<Partial<Host>>({ domain: "", target: "", scheme: "http" });

  // Edit Host Dialog
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [editingHost, setEditingHost] = useState<Host | null>(null);
  const [newLocation, setNewLocation] = useState<Location>({ path: "/", target: "", scheme: "http", rewrite: false });

  const handleAddHost = () => {
    if (!newHost.domain || !newHost.target) return toast.warning("Missing fields");
    addHostMutation.mutate(newHost, {
      onSuccess: () => {
        setIsAddOpen(false);
        setNewHost({ domain: "", target: "", scheme: "http" });
      }
    });
  };

  const handleDeleteHost = (domain: string) => {
    if (!confirm(`Delete ${domain}?`)) return;
    deleteHostMutation.mutate(domain);
  };

  const handleIssueCert = (domain: string) => {
    issueCertMutation.mutate({ domain, email: "admin@example.com" });
  };

  const handleAddLocation = () => {
    if (!editingHost || !newLocation.path || !newLocation.target) return;
    addLocationMutation.mutate({ domain: editingHost.domain, location: newLocation }, {
        onSuccess: () => {
            setNewLocation({ path: "/", target: "", scheme: "http", rewrite: false });
        }
    });
  };

  const handleDeleteLocation = (path: string) => {
    if (!editingHost) return;
    deleteLocationMutation.mutate({ domain: editingHost.domain, path });
  };

  // Update editingHost when hosts data changes
  useMemo(() => {
      if (editingHost && hosts) {
          const updated = hosts.find(h => h.domain === editingHost.domain);
          if (updated) setEditingHost(updated);
      }
  }, [hosts, editingHost]);

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
                <CardDescription>Manage your routing rules.</CardDescription>
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
                <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
                  <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" /> Add Host</Button></DialogTrigger>
                  <DialogContent>
                    <DialogHeader><DialogTitle>Add New Host</DialogTitle></DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <Label>Domain</Label>
                        <Input value={newHost.domain} onChange={e => setNewHost({...newHost, domain: e.target.value})} placeholder="example.com" />
                      </div>
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
                      <div className="grid gap-2">
                        <Label>Target</Label>
                        <Input value={newHost.target} onChange={e => setNewHost({...newHost, target: e.target.value})} placeholder="127.0.0.1:8080" />
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
              </div>
            </div>
       </CardHeader>
       <CardContent>
          <Table>
             <TableHeader>
               <TableRow>
                 <TableHead>Domain</TableHead>
                 <TableHead>Target</TableHead>
                 <TableHead>Locations</TableHead>
                 <TableHead className="text-right">Actions</TableHead>
               </TableRow>
             </TableHeader>
             <TableBody>
               {filteredHosts.map(host => (
                 <TableRow key={host.domain}>
                    <TableCell className="font-medium">{host.domain}</TableCell>
                    <TableCell>{host.target} <Badge variant="outline" className="ml-2 text-[10px]">{host.scheme}</Badge></TableCell>
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
                        <Button variant="ghost" size="sm" onClick={() => openEdit(host)}>
                          <Settings className="h-4 w-4 text-slate-500" />
                        </Button>
                        <Button variant="ghost" size="sm" onClick={() => handleIssueCert(host.domain)}>
                          <ShieldCheck className="h-4 w-4 text-blue-500" />
                        </Button>
                        <Button variant="ghost" size="sm" onClick={() => handleDeleteHost(host.domain)}>
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      </div>
                    </TableCell>
                 </TableRow>
               ))}
             </TableBody>
          </Table>
       </CardContent>

       {/* Edit Dialog */}
       <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
         <DialogContent className="max-w-2xl">
           <DialogHeader>
             <DialogTitle>Manage Locations for {editingHost?.domain}</DialogTitle>
           </DialogHeader>
           <div className="space-y-6 py-4">
             {/* Add Location Form */}
             <div className="grid gap-4 border-b pb-4">
                <div className="flex items-end gap-3">
                  <div className="grid gap-1 flex-1">
                    <Label>Path Prefix</Label>
                    <Input value={newLocation.path} onChange={e => setNewLocation({...newLocation, path: e.target.value})} placeholder="/api" />
                  </div>
                  <div className="grid gap-1 flex-1">
                    <Label>Target</Label>
                    <Input value={newLocation.target} onChange={e => setNewLocation({...newLocation, target: e.target.value})} placeholder="10.0.0.5:3000" />
                  </div>
                </div>
                <div className="flex items-end gap-3">
                  <div className="grid gap-1 w-[100px]">
                    <Label>Scheme</Label>
                    <Select value={newLocation.scheme} onValueChange={v => setNewLocation({...newLocation, scheme: v as any})}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="http">http</SelectItem>
                        <SelectItem value="https">https</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex items-center space-x-2 pb-2.5">
                      <input 
                        type="checkbox" 
                        id="rewrite" 
                        className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                        checked={newLocation.rewrite || false}
                        onChange={e => setNewLocation({...newLocation, rewrite: e.target.checked})}
                      />
                      <Label htmlFor="rewrite" className="cursor-pointer">Strip Path (Rewrite)</Label>
                  </div>
                  <Button className="ml-auto" onClick={handleAddLocation} disabled={addLocationMutation.isPending}>
                      <Plus className="h-4 w-4 mr-2" /> Add
                  </Button>
                </div>
             </div>

             {/* Locations List */}
             <div className="space-y-2">
               <Label>Configured Locations</Label>
               {editingHost?.locations?.length === 0 && (
                 <div className="text-sm text-muted-foreground italic">No extra paths configured. All traffic goes to default target.</div>
               )}
               {editingHost?.locations?.map((loc) => (
                 <div key={loc.path} className="flex items-center justify-between p-2 bg-slate-50 rounded border">
                   <div className="flex items-center gap-3">
                     <CornerDownRight className="h-4 w-4 text-slate-400" />
                     <Badge variant="outline">{loc.path}</Badge>
                     <span className="text-sm text-slate-500">forwards to</span>
                     <span className="font-mono text-sm">{loc.scheme}://{loc.target}</span>
                     {loc.rewrite && (
                        <Badge variant="secondary" className="text-[10px] bg-yellow-100 text-yellow-800 hover:bg-yellow-100">
                           Stripped
                        </Badge>
                     )}
                   </div>
                   <Button variant="ghost" size="sm" onClick={() => handleDeleteLocation(loc.path)} disabled={deleteLocationMutation.isPending}>
                     <Trash2 className="h-4 w-4 text-red-500" />
                   </Button>
                 </div>
               ))}
             </div>
           </div>
         </DialogContent>
       </Dialog>
    </Card>
  );
}
