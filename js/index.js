import('../dist/pkg').catch(console.error);

document.addEventListener('DOMContentLoaded', async () => {
  try {
    const wasm = await import('../dist/pkg');
    const app = new wasm.App({ bgcolor: '#202020', color: '#ffffff' });
    app.start();
  } catch (err) {
    console.error(err);
  }
});
