import { useEffect, useMemo, useRef, useState } from 'react';
import { Search } from 'lucide-react';
import { Dialog, DialogContent } from '../ui/dialog';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

export interface CommandPaletteItem {
  id: string;
  label: string;
  description?: string;
  category: string;
  shortcut?: string;
  keywords?: string[];
  action: () => void | Promise<void>;
}

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  commands: CommandPaletteItem[];
}

const normalize = (value: string) => value.trim().toLowerCase();

const matchesQuery = (command: CommandPaletteItem, query: string) => {
  if (!query) return true;
  const haystacks = [command.label, command.description ?? '', ...(command.keywords ?? [])];
  return haystacks.some((item) => normalize(item).includes(query));
};

const groupCommands = (commands: CommandPaletteItem[]) => {
  const map = new Map<string, CommandPaletteItem[]>();
  commands.forEach((command) => {
    const list = map.get(command.category) ?? [];
    list.push(command);
    map.set(command.category, list);
  });
  return Array.from(map.entries()).map(([category, items]) => ({
    category,
    items,
  }));
};

export function CommandPalette({ open, onOpenChange, commands }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);

  const normalizedQuery = useMemo(() => normalize(query), [query]);

  const filteredCommands = useMemo(() => {
    const filtered = commands.filter((command) => matchesQuery(command, normalizedQuery));
    return filtered;
  }, [commands, normalizedQuery]);

  const groupedCommands = useMemo(() => groupCommands(filteredCommands), [filteredCommands]);

  useEffect(() => {
    if (open) {
      setQuery('');
      setActiveIndex(0);
      const node = inputRef.current;
      if (node) {
        window.requestAnimationFrame(() => {
          node.focus();
          node.select();
        });
      }
    }
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [normalizedQuery]);

  const flatCommands = filteredCommands;

  const handleSelect = async (command: CommandPaletteItem) => {
    try {
      await Promise.resolve(command.action());
    } catch (error) {
      console.error('[CommandPalette] Failed to run command', error);
    } finally {
      onOpenChange(false);
    }
  };

  const handleKeyNavigation = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (!flatCommands.length) return;
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActiveIndex((index) => (index + 1) % flatCommands.length);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveIndex((index) => (index - 1 + flatCommands.length) % flatCommands.length);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const command = flatCommands[activeIndex];
      if (command) {
        void handleSelect(command);
      }
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl gap-0 p-0">
        <div className="flex items-center gap-2 border-b border-border/60 px-4 py-3">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Input
            ref={inputRef}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={handleKeyNavigation}
            placeholder="输入要执行的操作或页面名称..."
            className="flex-1 border-0 bg-transparent p-0 focus-visible:ring-0"
          />
        </div>

        <div className="max-h-80 overflow-y-auto px-2 py-2">
          {flatCommands.length === 0 ? (
            <p className="px-2 py-6 text-center text-sm text-muted-foreground">
              未找到匹配的命令，尝试调整搜索关键字。
            </p>
          ) : (
            groupedCommands.map(({ category, items }) => (
              <div key={category} className="py-1">
                <p className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {category}
                </p>
                <div className="space-y-1">
                  {items.map((command) => {
                    const commandIndex = flatCommands.indexOf(command);
                    const isActive = commandIndex === activeIndex;
                    return (
                      <Button
                        key={command.id}
                        type="button"
                        variant="ghost"
                        onClick={() => void handleSelect(command)}
                        className={cn(
                          'flex w-full items-center justify-between gap-3 rounded-xl border border-transparent px-3 py-2 text-left text-sm',
                          isActive
                            ? 'border-primary/40 bg-primary/10 text-primary shadow-sm'
                            : 'hover:border-border/60 hover:bg-muted/60',
                        )}
                      >
                        <span className="flex flex-col">
                          <span className="font-medium text-foreground">{command.label}</span>
                          {command.description ? (
                            <span className="text-xs text-muted-foreground">
                              {command.description}
                            </span>
                          ) : null}
                        </span>
                        {command.shortcut ? (
                          <kbd className="rounded border border-border/60 bg-muted px-2 py-1 text-[11px] font-medium text-muted-foreground">
                            {command.shortcut}
                          </kbd>
                        ) : null}
                      </Button>
                    );
                  })}
                </div>
              </div>
            ))
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default CommandPalette;
