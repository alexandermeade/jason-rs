-- Picks a random element safely
function random_pick(list)
    if not list or #list == 0 then return nil end
    return list[math.random(1, #list)]
end

-- Returns a random integer, defaults to 1 if max is nil or invalid
function random_int(max) 
    max = tonumber(max) or 1
    if max < 1 then max = 1 end
    return math.random(1, max)
end

-- Returns a random string of length n, defaults to 8
function random_string(n)
    n = tonumber(n) or 8
    local chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'
    local s = ''
    for i = 1, n do
        local idx = math.random(1, #chars)
        s = s .. chars:sub(idx, idx)
    end
    return s
end

-- Returns a random password of length n, defaults to 12
function random_password(n)
    n = tonumber(n) or 12
    local chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-='
    local pwd = ''
    for i = 1, n do
        local idx = math.random(1, #chars)
        pwd = pwd .. chars:sub(idx, idx)
    end
    return pwd
end

-- Returns a random domain name
function random_domain()
    local domains = {"example.com", "test.com", "demo.org", "site.net", "myapp.io"}
    local prefixes = {"www", "app", "api", "blog", ""}
    local prefix = random_pick(prefixes) or ""
    if prefix ~= "" then prefix = prefix .. "." end
    return prefix .. (random_pick(domains) or "example.com")
end

-- Returns a random email address
function random_email()
    local user = random_string(random_int(10)):lower()
    local domain = random_domain()
    return user .. "@" .. domain
end

-- Returns a random URL
function random_url()
    local protocols = {"http", "https"}
    local protocol = random_pick(protocols) or "http"
    local path_length = random_int(4)
    local path = ""
    for i = 1, path_length do
        path = path .. "/" .. random_string(random_int(8)):lower()
    end
    return protocol .. "://" .. random_domain() .. path
end

-- Returns a random IPv4 address
function random_ipv4()
    local parts = {}
    for i = 1, 4 do
        table.insert(parts, random_int(255)-1)  -- math.random(1,255)-1 gives 0-254
    end
    return table.concat(parts, ".")
end

function random_first_name()
    local first = {
        "Alex", "James", "John", "Michael", "Sarah", "Emily", "Hannah",
        "Laura", "Daniel", "David", "Chris", "Ryan", "Ethan", "Grace",
        "Liam", "Noah", "Olivia", "Ava", "Emma", "Mason"
    }

    local f = first[ math.random(#first) ]

    return f 
end

function random_last_name()
    local last = {
        "Smith", "Johnson", "Brown", "Garcia", "Miller", "Davis",
        "Martinez", "Lee", "Clark", "Walker", "Hall", "Young",
        "King", "Wright", "Lopez", "Hill", "Scott", "Green"
    }
    local l = last[ math.random(#last) ]
    return l
end


