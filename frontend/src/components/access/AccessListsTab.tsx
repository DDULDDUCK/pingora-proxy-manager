import { useState } from "react";
import { RefreshCw, Plus, Trash2, User, Globe, Shield, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useAccessLists, useAddAccessList, useDeleteAccessList, useAddClient, useRemoveClient, useAddIp, useRemoveIp } from "@/hooks/useAccessLists";
import { useAuth } from "@/App";
import type { AccessList } from "@/hooks/useAccessLists";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from "@/components/ui/table";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Card, CardContent, CardHeader, CardTitle, CardDescription,
} from "@/components/ui/card";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";

export function AccessListsTab() {
  const { t } = useTranslation();
  const { canManageHosts } = useAuth();
  const { data: accessLists, isLoading, refetch } = useAccessLists();
  const addListMutation = useAddAccessList();
  const deleteListMutation = useDeleteAccessList();
  const addClientMutation = useAddClient();
  const removeClientMutation = useRemoveClient();
  const addIpMutation = useAddIp();
  const removeIpMutation = useRemoveIp();

  const [isAddOpen, setIsAddOpen] = useState(false);
  const [newListName, setNewListName] = useState("");

  // Manage items dialog
  const [editingList, setEditingList] = useState<AccessList | null>(null);
  const [isManageOpen, setIsManageOpen] = useState(false);
  
  // New item states
  const [newClientUser, setNewClientUser] = useState("");
  const [newClientPass, setNewClientPass] = useState("");
  const [newIp, setNewIp] = useState("");
  const [newIpAction, setNewIpAction] = useState<"allow" | "deny">("allow");

  const handleAddList = () => {
    if (!newListName) return toast.warning(t('access.nameRequired'));
    addListMutation.mutate(newListName, {
        onSuccess: () => {
            setIsAddOpen(false);
            setNewListName("");
        }
    });
  };

  const handleDeleteList = (id: number) => {
      if (!confirm(t('access.deleteConfirm'))) return;
      deleteListMutation.mutate(id);
  };

  const openManage = (list: AccessList) => {
      setEditingList(list);
      setIsManageOpen(true);
  };

  const handleAddClient = () => {
      if (!editingList || !newClientUser || !newClientPass) return toast.warning(t('access.usernamePasswordRequired'));
      addClientMutation.mutate({ id: editingList.id, client: { username: newClientUser, password: newClientPass } }, {
          onSuccess: () => {
              setNewClientUser("");
              setNewClientPass("");
              // Optimistic update or refetch needed? 
              // Since we rely on parent query invalidation, we need to wait for refetch or close/reopen.
              // Ideally we should refetch specific list or invalidate accessLists.
              // The hook invalidates 'accessLists', so data should update automatically if we are observing it.
              // However, 'editingList' is a local state copy. We need to sync it.
              // Better approach: Use the ID to find the list from 'accessLists' in render.
          }
      });
  };

  const handleRemoveClient = (username: string) => {
      if (!editingList) return;
      removeClientMutation.mutate({ id: editingList.id, username });
  };

  const handleAddIp = () => {
      if (!editingList || !newIp) return toast.warning(t('access.ipAddressRequired'));
      addIpMutation.mutate({ id: editingList.id, ipRule: { ip: newIp, action: newIpAction } }, {
          onSuccess: () => {
              setNewIp("");
          }
      });
  };

  const handleRemoveIp = (ip: string) => {
      if (!editingList) return;
      removeIpMutation.mutate({ id: editingList.id, ip });
  };

  // Find the current editing list from the fresh data
  const currentEditingList = accessLists?.find(l => l.id === editingList?.id);

  return (
    <Card>
       <CardHeader className="pb-3">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <CardTitle>{t('access.title')}</CardTitle>
                <CardDescription>{t('access.description')}</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="outline" size="icon" onClick={() => refetch()} disabled={isLoading}>
                  <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
                </Button>
                {canManageHosts && (
                <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
                  <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" /> {t('access.addList')}</Button></DialogTrigger>
                  <DialogContent>
                    <DialogHeader><DialogTitle>{t('access.addList')}</DialogTitle>
                    <DialogDescription>
                    Create a new access list to restrict access to your hosts.
                    </DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <Label>{t('access.name')}</Label>
                        <Input value={newListName} onChange={e => setNewListName(e.target.value)} placeholder="Internal Admins" />
                      </div>
                    </div>
                    <DialogFooter>
                       <Button onClick={handleAddList} disabled={addListMutation.isPending}>{t('common.save')}</Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
                )}
              </div>
            </div>
       </CardHeader>
       <CardContent>
          <Table>
             <TableHeader>
               <TableRow>
                 <TableHead>{t('access.name')}</TableHead>
                 <TableHead>{t('access.clients')}</TableHead>
                 <TableHead>{t('access.ipRules')}</TableHead>
                 <TableHead className="text-right">{t('access.actions')}</TableHead>
               </TableRow>
             </TableHeader>
             <TableBody>
               {accessLists?.map(list => (
                 <TableRow key={list.id}>
                    <TableCell className="font-medium">{list.name}</TableCell>
                    <TableCell>
                        <div className="flex flex-wrap gap-1">
                            {list.clients.map(c => <Badge key={c.username} variant="outline" className="text-xs"><User className="h-3 w-3 mr-1"/>{c.username}</Badge>)}
                        </div>
                    </TableCell>
                    <TableCell>
                        <div className="flex flex-wrap gap-1">
                            {list.ips.map(ip => (
                                <Badge key={ip.ip} variant={ip.action === 'allow' ? 'default' : 'destructive'} className="text-xs">
                                    {ip.action === 'allow' ? <Shield className="h-3 w-3 mr-1"/> : <Globe className="h-3 w-3 mr-1"/>}
                                    {ip.ip}
                                </Badge>
                            ))}
                        </div>
                    </TableCell>
                    <TableCell className="text-right">
                        {canManageHosts && (
                          <Button variant="outline" size="sm" className="mr-2" onClick={() => openManage(list)}>
                              {t('access.manage')}
                          </Button>
                        )}
                        {canManageHosts && (
                          <Button variant="ghost" size="sm" onClick={() => handleDeleteList(list.id)}>
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        )}
                    </TableCell>
                 </TableRow>
               ))}
               {(!accessLists || accessLists.length === 0) && (
                   <TableRow>
                       <TableCell colSpan={4} className="text-center text-muted-foreground h-24">No access lists found.</TableCell>
                   </TableRow>
               )}
             </TableBody>
          </Table>
       </CardContent>

       {/* Manage Items Dialog */}
       <Dialog open={isManageOpen} onOpenChange={setIsManageOpen}>
           <DialogContent className="max-w-3xl">
               <DialogHeader>
                   <DialogTitle>{t('access.manageItems')} - {editingList?.name}</DialogTitle>
               </DialogHeader>
               <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                   {/* Clients Column */}
                   <div className="space-y-4">
                       <div className="flex items-center justify-between border-b pb-2">
                           <h3 className="font-semibold">{t('access.basicAuth')}</h3>
                       </div>
                       <div className="flex gap-2 items-end">
                           <div className="grid gap-1 flex-1">
                               <Label className="text-xs">{t('access.username')}</Label>
                               <Input value={newClientUser} onChange={e => setNewClientUser(e.target.value)} />
                           </div>
                           <div className="grid gap-1 flex-1">
                               <Label className="text-xs">{t('access.password')}</Label>
                               <Input type="password" value={newClientPass} onChange={e => setNewClientPass(e.target.value)} />
                           </div>
                           <Button size="sm" onClick={handleAddClient} disabled={addClientMutation.isPending}><Plus className="h-4 w-4"/></Button>
                       </div>
                       <div className="space-y-2 max-h-[300px] overflow-y-auto">
                           {currentEditingList?.clients.map(client => (
                               <div key={client.username} className="flex items-center justify-between p-2 bg-slate-50 rounded text-sm">
                                   <span className="flex items-center"><User className="h-3 w-3 mr-2 text-slate-500"/>{client.username}</span>
                                   <Button variant="ghost" size="sm" className="h-6 w-6 p-0" onClick={() => handleRemoveClient(client.username)} disabled={removeClientMutation.isPending}>
                                       <X className="h-3 w-3 text-red-500"/>
                                   </Button>
                               </div>
                           ))}
                           {currentEditingList?.clients.length === 0 && <p className="text-xs text-muted-foreground text-center py-4">No users</p>}
                       </div>
                   </div>

                   {/* IPs Column */}
                   <div className="space-y-4">
                       <div className="flex items-center justify-between border-b pb-2">
                           <h3 className="font-semibold">{t('access.ipAddressRules')}</h3>
                       </div>
                       <div className="flex gap-2 items-end">
                           <div className="grid gap-1 flex-1">
                               <Label className="text-xs">{t('access.ipAddress')}</Label>
                               <Input value={newIp} onChange={e => setNewIp(e.target.value)} placeholder="1.2.3.4" />
                           </div>
                           <div className="grid gap-1 w-[80px]">
                               <Label className="text-xs">{t('access.action')}</Label>
                               <Select value={newIpAction} onValueChange={v => setNewIpAction(v as any)}>
                                   <SelectTrigger className="h-10"><SelectValue/></SelectTrigger>
                                   <SelectContent>
                                       <SelectItem value="allow">{t('access.allow')}</SelectItem>
                                       <SelectItem value="deny">{t('access.deny')}</SelectItem>
                                   </SelectContent>
                               </Select>
                           </div>
                           <Button size="sm" onClick={handleAddIp} disabled={addIpMutation.isPending}><Plus className="h-4 w-4"/></Button>
                       </div>
                       <div className="space-y-2 max-h-[300px] overflow-y-auto">
                           {currentEditingList?.ips.map(ip => (
                               <div key={ip.ip} className="flex items-center justify-between p-2 bg-slate-50 rounded text-sm">
                                   <span className="flex items-center">
                                       {ip.action === 'allow' ? <Shield className="h-3 w-3 mr-2 text-green-600"/> : <Globe className="h-3 w-3 mr-2 text-red-500"/>}
                                       <span className={ip.action === 'deny' ? 'text-red-700' : 'text-green-700'}>{ip.ip}</span>
                                   </span>
                                   <Button variant="ghost" size="sm" className="h-6 w-6 p-0" onClick={() => handleRemoveIp(ip.ip)} disabled={removeIpMutation.isPending}>
                                       <X className="h-3 w-3 text-red-500"/>
                                   </Button>
                               </div>
                           ))}
                           {currentEditingList?.ips.length === 0 && <p className="text-xs text-muted-foreground text-center py-4">No IP rules</p>}
                       </div>
                   </div>
               </div>
           </DialogContent>
       </Dialog>
    </Card>
  );
}
