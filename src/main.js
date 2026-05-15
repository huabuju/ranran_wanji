import { createApp } from "vue";
import "./assets/main.scss";
import App from "./App.vue";
import router from "./router";
import ElementPlus from "element-plus";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import "dayjs/locale/zh-cn";

const app = createApp(App);

app
  .use(ElementPlus, {
    locale: zhCn,
  })
  .use(router);

app.mount("#app");
