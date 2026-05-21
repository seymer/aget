local get_state = ya.sync(function()
	local h = cx.active.current.hovered
	local cwd = cx.active.current.cwd
	return h and tostring(h.url) or nil, tostring(cwd)
end)

return {
	entry = function(_, job)
		local action = job.args[1]
		local url, cwd = get_state()

		if not url then
			ya.notify({ title = "aget", content = "No file selected", level = "warn", timeout = 3 })
			return
		end

		if action == "seal" or action == "seal-keep" then
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

			local cmd = Command("aget"):arg("seal"):arg("--passphrase")
			if action == "seal-keep" then
				cmd = cmd:arg("--keep")
			end
			local child = cmd:arg(url)
				:stdin(Command.PIPED)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:spawn()

			child:write_all(pass .. "\n")
			child:flush()

			local output = child:wait_with_output()
			if output.status.success then
				local msg = action == "seal-keep" and "Sealed ✓ (kept)" or "Sealed ✓"
				ya.notify({ title = "aget", content = msg, level = "info", timeout = 3 })
			else
				ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
			end

		elseif action == "open" or action == "peek" then
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

			if action == "peek" then
				-- Decrypt to /tmp, view, then securely delete (block mode)
				ya.emit("shell", {
					"echo " .. ya.quote(pass) .. " | aget open " .. ya.quote(url),
					block = true,
				})
			else
				-- Decrypt to current directory, keep the file
				local child = Command("aget")
					:arg("open"):arg("--output"):arg(cwd)
					:arg(url)
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
					ya.notify({ title = "aget", content = "Decrypted ✓", level = "info", timeout = 3 })
				else
					ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
				end
			end

		elseif action == "destroy" then
			local confirm, event = ya.input({
				title = "Type 'yes' to destroy " .. url:match("[^/]+$") .. ":",
				pos = { "center", w = 50 },
			})
			if event ~= 1 or confirm ~= "yes" then return end

			local output = Command("aget")
				:arg("destroy"):arg("--no-confirm"):arg(url)
				:stdout(Command.PIPED)
				:stderr(Command.PIPED)
				:output()

			if output.status.success then
				ya.notify({ title = "aget", content = "Destroyed ✓", level = "info", timeout = 3 })
			else
				ya.notify({ title = "aget", content = output.stderr, level = "error", timeout = 5 })
			end
		end
	end,
}
