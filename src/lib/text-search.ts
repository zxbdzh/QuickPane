export function matchesTextQuery(query: string, ...values: string[]) {
  const normalized = query.trim().toLowerCase();
  return (
    !normalized ||
    values.some((value) => value.toLowerCase().includes(normalized))
  );
}
