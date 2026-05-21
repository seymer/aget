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
				title = "Passphrase:",
				pos = { "center", w = 40 },
				obscure = true,
			})
			if event ~= 1 then return end

			local confirm, event2 = ya.input({
				title = "Confirm passphrase:",
				pos = { "center", w = 40 },
				obscure = true,
			})
			if event2 ~= 1 then return end
			if pass ~= confirm then
				ya.notify({ title = "aget", content = "Passphrases don't match", level = "error", timeout = 3 })
				return
			end

			local child = Command("aget")
				:arg("seal"):arg("--passphrase"):arg(url)
				:stdin(Command.PIPED)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:spawn()

			child:write_all(pass .. "\n")
			child:flush()

			local output = child:wait_with_output()
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
				title = "Passphrase:",
				pos = { "center", w = 40 },
				obscure = true,
			})
			if event ~= 1 then return end

			local child = Command("aget")
				:arg("open"):arg("--no-wait"):arg(url)
				:stdin(Command.PIPED)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:spawn()

			child:write_all(pass .. "\n")
			child:flush()

			local output = child:wait_with_output()
			if output.status.success then
				local path = output.stdout:gsub("%s+$", "")
				if path ~= "" then
					ya.emit("reveal", { Url(path) })
				end
				ya.notify({ title = "aget", content = "Decrypted ✓", level = "info", timeout = 5 })
			else
				ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
			end
		end
	end,
}
