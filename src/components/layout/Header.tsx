import { Laptop, Menu, Moon, Sun } from 'lucide-react';
import { ReactNode } from 'react';
import { useTheme, type ThemeMode } from '../../providers/theme-provider';

interface HeaderProps {
  onOpenMobileNav: () => void;
  actions?: ReactNode;
  phaseLabel?: string;
  phaseStatus?: ReactNode;
}

export function Header({ onOpenMobileNav, actions, phaseLabel, phaseStatus }: HeaderProps) {
  return (
    <header className="sticky top-0 z-30 border-b border-border/60 bg-background/75 backdrop-blur">
      <div className="flex h-14 items-center gap-3 px-4 sm:px-6">
        <button
          type="button"
          aria-label="打开导航"
          className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-border/60 text-foreground transition hover:bg-muted/70 lg:hidden"
          onClick={onOpenMobileNav}
        >
          <Menu className="h-4 w-4" />
        </button>

        <div className="hidden items-center gap-2 lg:flex">
          <span className="rounded-full bg-gradient-to-r from-primary/10 to-primary/5 px-4 py-1 text-xs font-semibold uppercase tracking-[0.25em] text-primary">
            CogniCal
          </span>
        </div>

        <div className="flex flex-1 items-center gap-3">
          <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
            <span className="text-foreground/90">{phaseLabel ?? '智能任务与时间管理'}</span>
            {phaseStatus}
          </div>
        </div>

        {actions ? <div className="flex items-center gap-2">{actions}</div> : null}

        <ThemeToggle />
      </div>
    </header>
  );
}

function ThemeToggle() {
  const { theme, resolvedTheme, setTheme } = useTheme();

  const modes: ThemeMode[] = ['light', 'dark', 'system'];
  const currentIndex = Math.max(0, modes.indexOf(theme));
  const nextMode = modes[(currentIndex + 1) % modes.length];
  const labels: Record<ThemeMode, string> = {
    light: '浅色主题',
    dark: '深色主题',
    system: '跟随系统',
  };

  const handleToggle = () => {
    setTheme(nextMode);
  };

  const icon = {
    light: <Sun className="h-[18px] w-[18px]" />,
    dark: <Moon className="h-[18px] w-[18px]" />,
    system: <Laptop className="h-[18px] w-[18px]" />,
  }[theme === 'system' ? 'system' : theme];

  return (
    <button
      type="button"
      onClick={handleToggle}
      aria-label={`切换主题：当前 ${labels[theme]}，下一步 ${labels[nextMode]}`}
      title={`当前 ${labels[theme]} · 点击切换为 ${labels[nextMode]}`}
      className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-border/60 text-foreground transition hover:bg-muted/70"
    >
      {icon}
      <span className="sr-only">{`当前主题：${resolvedTheme}`}</span>
    </button>
  );
}
