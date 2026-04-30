function run() {
    const system = Application("System Events");
    const frontApp = system.processes.whose({frontmost: true})[0].name();

    try {
        if (["Google Chrome", "Brave Browser"].includes(frontApp)) {
            const browser = Application(frontApp);
            return frontApp + " - " + browser.windows[0].activeTab().url();
        } else if (frontApp === "Safari") {
            const safari = Application("Safari");
            return frontApp + " - " + safari.documents[0].url();
        } else if (frontApp === "Arc") {
            const arc = Application("Arc");
            return frontApp + " - " + arc.windows[0].activeTab().url();
        }
        return frontApp;
    } catch (e) {
        return frontApp;
    }
}
