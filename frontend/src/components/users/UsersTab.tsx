import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Edit2, Shield, User, Eye, Loader2 } from "lucide-react";
import {
  useUsers,
  useCreateUser,
  useUpdateUser,
  useDeleteUser,
} from "@/hooks/useUsers";
import type { User as UserType } from "@/hooks/useUsers";
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const roleConfig = {
  admin: { label: "Admin", color: "destructive" as const, icon: Shield },
  operator: { label: "Operator", color: "default" as const, icon: Edit2 },
  viewer: { label: "Viewer", color: "secondary" as const, icon: Eye },
};

export function UsersTab() {
  const { t } = useTranslation();
  const { data: users, isLoading } = useUsers();
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const deleteUser = useDeleteUser();

  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<UserType | null>(null);

  // Create form
  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState("viewer");

  // Edit form
  const [editPassword, setEditPassword] = useState("");
  const [editRole, setEditRole] = useState("");

  const handleCreate = async () => {
    if (!newUsername || !newPassword) {
      toast.error(t('users.usernamePasswordRequired'));
      return;
    }

    try {
      await createUser.mutateAsync({
        username: newUsername,
        password: newPassword,
        role: newRole,
      });
      toast.success(t('users.userCreated', { username: newUsername }));
      setCreateOpen(false);
      setNewUsername("");
      setNewPassword("");
      setNewRole("viewer");
    } catch {
      toast.error(t('users.createUserFailed'));
    }
  };

  const handleEdit = async () => {
    if (!editingUser) return;

    const data: { password?: string; role?: string } = {};
    if (editPassword) data.password = editPassword;
    if (editRole && editRole !== editingUser.role) data.role = editRole;

    if (Object.keys(data).length === 0) {
      toast.info(t('users.noChangesToSave'));
      return;
    }

    try {
      await updateUser.mutateAsync({ id: editingUser.id, data });
      toast.success(t('users.userUpdated', { username: editingUser.username }));
      setEditOpen(false);
      setEditingUser(null);
      setEditPassword("");
      setEditRole("");
    } catch {
      toast.error(t('users.updateUserFailed'));
    }
  };

  const handleDelete = async (user: UserType) => {
    if (!confirm(t('users.deleteConfirm', { username: user.username }))) {
      return;
    }

    try {
      await deleteUser.mutateAsync(user.id);
      toast.success(t('users.userDeleted'));
    } catch {
      toast.error(t('users.deleteUserFailed'));
    }
  };

  const openEdit = (user: UserType) => {
    setEditingUser(user);
    setEditRole(user.role);
    setEditPassword("");
    setEditOpen(true);
  };

  const formatDate = (timestamp: number | null): string => {
    if (!timestamp) return t('users.never');
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            {t('users.title')}
          </CardTitle>
          <CardDescription>
            {t('users.description')}
          </CardDescription>
        </div>

        <Dialog open={createOpen} onOpenChange={setCreateOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> {t('users.addUser')}
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{t('users.addUser')}</DialogTitle>
              <DialogDescription>
                Add a new user to the system
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="username">{t('users.username')}</Label>
                <Input
                  id="username"
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                  placeholder="Enter username"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">{t('users.password')}</Label>
                <Input
                  id="password"
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder="Enter password"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="role">{t('users.role')}</Label>
                <Select value={newRole} onValueChange={setNewRole}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select role" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="admin">{t('users.admin')} - Full access</SelectItem>
                    <SelectItem value="operator">
                      {t('users.operator')} - Manage hosts & streams
                    </SelectItem>
                    <SelectItem value="viewer">{t('users.viewer')} - Read only</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setCreateOpen(false)}>
                {t('users.cancel')}
              </Button>
              <Button onClick={handleCreate} disabled={createUser.isPending}>
                {createUser.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {t('users.addUser')}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </CardHeader>

      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('users.username')}</TableHead>
              <TableHead>{t('users.role')}</TableHead>
              <TableHead>Created</TableHead>
              <TableHead>{t('users.lastLogin')}</TableHead>
              <TableHead className="text-right">{t('users.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users?.map((user) => {
              const role = roleConfig[user.role as keyof typeof roleConfig] || roleConfig.viewer;
              const RoleIcon = role.icon;
              
              return (
                <TableRow key={user.id}>
                  <TableCell className="font-medium">{user.username}</TableCell>
                  <TableCell>
                    <Badge variant={role.color} className="flex items-center gap-1 w-fit">
                      <RoleIcon className="h-3 w-3" />
                      {t(`users.${user.role}`)}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {formatDate(user.created_at)}
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {formatDate(user.last_login)}
                  </TableCell>
                  <TableCell className="text-right space-x-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => openEdit(user)}
                    >
                      <Edit2 className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDelete(user)}
                      className="text-destructive hover:text-destructive"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              );
            })}
            {(!users || users.length === 0) && (
              <TableRow>
                <TableCell colSpan={5} className="text-center text-muted-foreground">
                  No users found
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>

      {/* Edit Dialog */}
      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('users.editUser')}</DialogTitle>
            <DialogDescription>
              Update user "{editingUser?.username}"
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit-role">{t('users.role')}</Label>
              <Select value={editRole} onValueChange={setEditRole}>
                <SelectTrigger>
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="admin">{t('users.admin')} - Full access</SelectItem>
                  <SelectItem value="operator">
                    {t('users.operator')} - Manage hosts & streams
                  </SelectItem>
                  <SelectItem value="viewer">{t('users.viewer')} - Read only</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-password">{t('users.newPassword')}</Label>
              <Input
                id="edit-password"
                type="password"
                value={editPassword}
                onChange={(e) => setEditPassword(e.target.value)}
                placeholder={t('users.leaveBlank')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditOpen(false)}>
              {t('users.cancel')}
            </Button>
            <Button onClick={handleEdit} disabled={updateUser.isPending}>
              {updateUser.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {t('users.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  );
}