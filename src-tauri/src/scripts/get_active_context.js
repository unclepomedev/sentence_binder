function stripUrl(url) {
    if (!url) return "";
    const strUrl = String(url);
    return strUrl.split('?')[0].split('#')[0];
}

function run() {
    const system = Application("System Events");
    const frontApp = system.processes.whose({frontmost: true})[0].name();

    try {
        if (["Google Chrome", "Brave Browser"].includes(frontApp)) {
            const browser = Application(frontApp);
            return frontApp + " - " + stripUrl(browser.windows[0].activeTab().url());
        } else if (frontApp === "Safari") {
            const safari = Application("Safari");
            return frontApp + " - " + stripUrl(safari.documents[0].url());
        } else if (frontApp === "Arc") {
            const arc = Application("Arc");
            return frontApp + " - " + stripUrl(arc.windows[0].activeTab().url());
        }
        return frontApp;
    } catch (e) {
        return frontApp;
    }
}
