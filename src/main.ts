import { createApp } from "vue";
import { createPinia } from "pinia";
import VueKonva from "vue-konva";
import App from "./App.vue";
import { router } from "./router";
import { syncThemeWithSystem } from "./lib/theme";
import "./style.css";

// Appliqué avant le mount pour éviter un flash du mauvais thème au démarrage.
syncThemeWithSystem();

const app = createApp(App);

// vue-konva's generic shape components (Group, Circle, ...) render a bare
// child list with no single DOM root, so Vue's dev-only "extraneous listener"
// heuristic fires whenever we bind @click/@mouseenter/@mouseleave on them —
// even though vue-konva actually wires those listeners itself by reading
// vnode.props and calling Konva's own .on(), independently of that heuristic.
// Harmless and dev-only (stripped in production); just noisy, so filter it.
app.config.warnHandler = (msg, _instance, trace) => {
  if (msg.includes("Extraneous non-emits event listeners")) return;
  console.warn(`[Vue warn]: ${msg}${trace}`);
};

app.use(createPinia()).use(router).use(VueKonva).mount("#app");
