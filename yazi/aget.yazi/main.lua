local get_url = ya.sync(function()
	local h = cx.active.current.hovered
	return h and tostring(h.url) or nil
end)

return {
	entry = function(_, job)
		local action = job.args[1]
		local url = get_url()

		if not url then
			ya.notify({ title = "aget", content = "No file selected", level = "warn", timeout = 3 })
			return
		end

		if action == "seal" then
			if url:match("%.age$") then
				ya.notify({ title = "aget", content = "Already encrypted", level = "warn", timeout = 3 })
				return
			end

			local pass, event = ya.input({
				title = "Passphrase (seal):",
				pos = { "center", w = 40 },
				obscure = true,
			})
			if event ~= 1 then return end

			local output = Command("aget")
				:arg("seal"):arg("--passphrase"):arg(url)
				:env("AGET_PASSPHRASE", pass)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:output()

			if output.status.success then
				ya.notify({ title = "aget", content = "Sealed ✓", level = "info", timeout = 3 })
			else
				ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
			end

		elseif action == "open" then
			if not url:match("%.age$") then
				ya.notify({ title = "aget", content = "Not an .age file", level = "warn", timeout = 3 })
				return
			end

			local pass, event = ya.input({
				title = "Passphrase (open):",
				pos = { "center", w = 40 },
				obscure = true,
			})
			if event ~= 1 then return end

			local output = Command("aget")
				:arg("open"):arg("--no-wait"):arg(url)
				:env("AGET_PASSPHRASE", pass)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:output()

			if output.status.success then
				-- stdout contains the decrypted file path
				local path = output.stdout:gsub("%s+$", "")
				if path ~= "" then
					ya.emit("reveal", { Url(path) })
					ya.notify({ title = "aget", content = "Decrypted ✓ (press Enter in notification to cleanup)", level = "info", timeout = 10 })
					-- Clean up after a delay to give user time to view
					local _, confirm_event = ya.input({
						title = "Press Enter to securely delete plaintext:",
						pos = { "center", w = 50 },
					})
					Command("aget"):arg("cleanup"):arg(path):output()
					ya.notify({ title = "aget", content = "Plaintext cleaned ✓", level = "info", timeout = 3 })
				end
			else
				ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
			end
		end
	end,
}
