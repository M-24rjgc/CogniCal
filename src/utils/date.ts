const ISO_DATE_PREFIX = /^(\d{4}-\d{2}-\d{2})/;
const DATE_ONLY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;

export function formatDateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function extractDateKey(value: string | Date | null | undefined): string | null {
  if (value instanceof Date) {
    return formatDateKey(value);
  }
  if (typeof value !== 'string') {
    return null;
  }

  const trimmed = value.trim();
  const isoMatch = ISO_DATE_PREFIX.exec(trimmed);
  if (isoMatch) {
    return isoMatch[1];
  }

  const parsed = new Date(trimmed);
  if (!Number.isNaN(parsed.getTime())) {
    return formatDateKey(parsed);
  }

  return null;
}

export function parseDateTime(value: string | Date | null | undefined): Date {
  if (value instanceof Date) {
    return new Date(value.getTime());
  }
  if (typeof value !== 'string') {
    return new Date();
  }

  const trimmed = value.trim();
  const dateOnlyMatch = DATE_ONLY_PATTERN.exec(trimmed);
  if (dateOnlyMatch) {
    const [, year, month, day] = dateOnlyMatch;
    return new Date(Number(year), Number(month) - 1, Number(day));
  }

  const parsed = new Date(trimmed);
  if (!Number.isNaN(parsed.getTime())) {
    return parsed;
  }

  const fallbackKey = extractDateKey(trimmed);
  if (fallbackKey) {
    const [year, month, day] = fallbackKey.split('-').map(Number);
    return new Date(year, month - 1, day);
  }

  return new Date();
}

export function isSameDay(
  first: Date | null | undefined,
  second: Date | null | undefined,
): boolean {
  if (!first || !second) {
    return false;
  }
  return (
    first.getFullYear() === second.getFullYear() &&
    first.getMonth() === second.getMonth() &&
    first.getDate() === second.getDate()
  );
}
