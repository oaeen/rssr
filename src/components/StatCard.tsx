type StatCardProps = {
  title: string;
  value: string;
};

export function StatCard({ title, value }: StatCardProps) {
  return (
    <article className="stat-card">
      <p className="stat-title">{title}</p>
      <p className="stat-value">{value}</p>
    </article>
  );
}
