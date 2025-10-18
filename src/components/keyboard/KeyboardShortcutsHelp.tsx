import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';

export interface ShortcutDescriptor {
  keys: string;
  description: string;
}

export interface ShortcutGroup {
  title: string;
  shortcuts: ShortcutDescriptor[];
}

interface KeyboardShortcutsHelpProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  groups: ShortcutGroup[];
}

export function KeyboardShortcutsHelp({ open, onOpenChange, groups }: KeyboardShortcutsHelpProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>键盘快捷键</DialogTitle>
          <p className="text-sm text-muted-foreground">
            借助快捷键快速完成常用操作。按 Esc 可在任意面板中退出。
          </p>
        </DialogHeader>
        <div className="grid gap-6 md:grid-cols-2">
          {groups.map((group) => (
            <div
              key={group.title}
              className="space-y-3 rounded-2xl border border-border/60 bg-muted/20 p-4"
            >
              <h3 className="text-sm font-semibold text-foreground">{group.title}</h3>
              <div className="space-y-2">
                {group.shortcuts.map((shortcut) => (
                  <div
                    key={`${group.title}-${shortcut.keys}`}
                    className="flex items-start justify-between gap-3 rounded-xl bg-background/80 px-3 py-2 text-sm"
                  >
                    <span className="text-muted-foreground">{shortcut.description}</span>
                    <kbd className="whitespace-nowrap rounded border border-border/60 bg-muted px-2 py-1 text-[11px] font-medium text-muted-foreground">
                      {shortcut.keys}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default KeyboardShortcutsHelp;
