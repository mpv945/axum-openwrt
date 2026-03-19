async function loadApi() {
    try {
        const res = await fetch('/api/hello');
        const text = await res.text();

        document.getElementById('msg').innerText = text;
    } catch (e) {
        document.getElementById('msg').innerText = 'API Error';
    }
}

// 页面加载时执行
window.onload = () => {
    document.getElementById('msg').innerText = 'Ready';
};