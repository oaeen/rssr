export function ReaderPage() {
  return (
    <section className="page-grid">
      <article className="page-card">
        <h2>阅读面板</h2>
        <p>下一步会实现：文章列表、详情、已读状态、搜索和过滤。</p>
      </article>
      <article className="page-card">
        <h3>性能目标</h3>
        <p>列表虚拟化与增量加载，保证高条目量场景下流畅滚动。</p>
      </article>
    </section>
  );
}
