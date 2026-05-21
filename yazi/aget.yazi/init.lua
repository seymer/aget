--- @module aget
-- Yazi plugin for encrypting/decrypting files via aget

local function entry(_, job)
	local action = job.args[1]

	local h = cx.active.current.hovered
	if not h then
		ya.notify({ title = "aget", content = "No file selected", level = "warn", timeout = 3 })
		return
	end

	local url = tostring(h.url)

	if action == "seal" then
		ya.manager_emit("shell", {
			"aget seal --passphrase " .. ya.quote(url),
			block = true,
			confirm = true,
		})
	elseif action == "open" then
		if not url:match("%.age$") then
			ya.notify({ title = "aget", content = "Not an .age file", level = "warn", timeout = 3 })
			return
		end
		ya.manager_emit("shell", {
			"aget open " .. ya.quote(url),
			block = true,
			confirm = true,
		})
	end
end

return { entry = entry }
