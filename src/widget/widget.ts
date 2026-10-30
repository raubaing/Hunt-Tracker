import "../styles.css";
import { mount } from "svelte";
import Widget from "./Widget.svelte";

export default mount(Widget, { target: document.getElementById("app")! });
