import "./app.css";
import { initI18n } from "./lib/i18n.svelte";
import App from "./App.svelte";
import { mount } from "svelte";

async function boot() {
  await initI18n();
  mount(App, { target: document.getElementById("app")! });
}

boot();
