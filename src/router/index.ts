import { createRouter, createWebHashHistory } from "vue-router";
import ClusterEditor from "@/pages/ClusterEditor.vue";
import Equipment from "@/pages/Equipment.vue";
import Aggregate from "@/pages/Aggregate.vue";
import Toolbox from "@/pages/Toolbox.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/clusters" },
    { path: "/clusters", name: "clusters", component: ClusterEditor },
    { path: "/equipment", name: "equipment", component: Equipment },
    { path: "/aggregate", name: "aggregate", component: Aggregate },
    { path: "/toolbox", name: "toolbox", component: Toolbox },
  ],
});
