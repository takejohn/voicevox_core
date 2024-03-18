import test from "ava";
import { SupportedDevices } from "../index";

test("サポートされているデバイス情報", (t) => {
    const supportedDevices = SupportedDevices.create();
    t.is(typeof supportedDevices, "object");
    t.not(supportedDevices, null);
    t.is(supportedDevices.cpu, true);
});