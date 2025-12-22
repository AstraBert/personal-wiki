function copyLink() {
    const linkInput = document.getElementById('wikiLink');
    linkInput.select();
    document.execCommand('copy');
            
    const btn = document.getElementById('copyButton');
    const originalText = btn.textContent;
    btn.textContent = 'Copied!';
    setTimeout(() => {
        btn.textContent = originalText;
    }, 2000);
}

document.getElementById('createWiki').addEventListener('click', async () => {
    const btn = document.getElementById('createWiki');
    btn.textContent = "Creating wiki...";
    btn.classList.add("disabled");
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const wikiText = document.getElementById('wiki').value;
    if (username && wikiText && password) {
        const response = await fetch("/wikis", {
                method: "POST",
                body: JSON.stringify({ "username": username, "content": wikiText, "password": password }),
                headers: {"Content-Type": "application/json"},
            }
        )
        if (response.ok) {
            const jsonResponse = await response.json()
            // validate
            if ("success" in jsonResponse && "error" in jsonResponse && "url" in jsonResponse) {
                if (jsonResponse.success) {
                    btn.textContent = "Created Wiki!";
                    setTimeout(() => {
                        btn.textContent = "Create Wiki";
                    }, 2000);
                    btn.classList.remove("disabled");
                    document.getElementById('wikiLink').value = `https://personalwiki.com.de/wikis/${username}`;
                    document.getElementById('linkContainer').classList.remove('hidden');
                } else {
                    btn.textContent = "Create Wiki";
                    btn.classList.remove("disabled");
                    document.getElementById('wikiLink').value = `An error occurred: ${jsonResponse.error}`;
                    document.getElementById('linkContainer').classList.remove('hidden');
                    document.getElementById('copyButton').classList.add('hidden');
                }
            }
        }
    } else {
        btn.textContent = "Create Wiki";
        btn.classList.remove("disabled");
        document.getElementById('wikiLink').value = `Please make sure to have filled out the username, the password and the wiki text fields`;
        document.getElementById('linkContainer').classList.remove('hidden');
        document.getElementById('copyButton').classList.add('hidden');
    }
});

document.getElementById('updateWiki').addEventListener('click', async () => {
    const btn = document.getElementById('updateWiki');
    btn.textContent = "Updating wiki...";
    btn.classList.add("disabled");
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const wikiText = document.getElementById('wiki').value;
    if (username && wikiText && password) {
        const response = await fetch("/wikis", {
                method: "PATCH",
                body: JSON.stringify({ "username": username, "content": wikiText, "password": password }),
                headers: {"Content-Type": "application/json"},
            }
        )
        if (response.ok) {
            const jsonResponse = await response.json()
            // validate
            if ("success" in jsonResponse && "error" in jsonResponse && "url" in jsonResponse) {
                if (jsonResponse.success) {
                    btn.textContent = "Updated Wiki!";
                    setTimeout(() => {
                        btn.textContent = "Update Wiki";
                    }, 2000);
                    btn.classList.remove("disabled");
                    document.getElementById('wikiLink').value = `https://personalwiki.com.de/wikis/${username}`;
                    document.getElementById('linkContainer').classList.remove('hidden');
                } else {
                    btn.textContent = "Update Wiki";
                    btn.classList.remove("disabled");
                    document.getElementById('wikiLink').value = `An error occurred: ${jsonResponse.error}`;
                    document.getElementById('linkContainer').classList.remove('hidden');
                    document.getElementById('copyButton').classList.add('hidden');
                }
            }
        }
    }  else {
        btn.textContent = "Update Wiki";
        btn.classList.remove("disabled");
        document.getElementById('wikiLink').value = `Please make sure to have filled out the username, the password and the wiki text fields`;
        document.getElementById('linkContainer').classList.remove('hidden');
        document.getElementById('copyButton').classList.add('hidden');
    }
});

document.getElementById('deleteWiki').addEventListener('click', async () => {
    const btn = document.getElementById('deleteWiki');
    btn.textContent = "Deleting wiki...";
    btn.classList.add("disabled");
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    if (username && password) {
        const response = await fetch("/wikis", {
                method: "DELETE",
                body: JSON.stringify({ "username": username, "password": password }),
                headers: {"Content-Type": "application/json"},
            }
        )
        if (response.ok) {
            const jsonResponse = await response.json()
            // validate
            if ("success" in jsonResponse && "error" in jsonResponse) {
                if (jsonResponse.success) {
                    btn.textContent = "Deleted Wiki!";
                    setTimeout(() => {
                        btn.textContent = "Delete Wiki";
                    }, 2000);
                    btn.classList.remove("disabled");
                } else {
                    btn.textContent = "Delete Wiki";
                    btn.classList.remove("disabled");
                    document.getElementById('wikiLink').value = `An error occurred: ${jsonResponse.error}`;
                    document.getElementById('linkContainer').classList.remove('hidden');
                    document.getElementById('copyButton').classList.add('hidden');
                }
            }
        }
    }  else {
        btn.textContent = "Delete Wiki";
        btn.classList.remove("disabled");
        document.getElementById('wikiLink').value = `Please make sure to have filled out the username, the password and the wiki text fields`;
        document.getElementById('linkContainer').classList.remove('hidden');
        document.getElementById('copyButton').classList.add('hidden');
    }
});