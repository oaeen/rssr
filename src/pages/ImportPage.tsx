export function ImportPage() {
  return (
    <section className="page-grid">
      <article className="page-card">
        <h2>导入中心</h2>
        <p>下一步会实现：OPML、XML、URL 列表与 JSON Feed 导入。</p>
      </article>
      <article className="page-card">
        <h3>导入策略</h3>
        <p>默认幂等导入，重复订阅不会重复创建。</p>
      </article>
    </section>
  );
}
