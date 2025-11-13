
function random_name()
    local names = {
        "James", "Mary", "Robert", "Patricia", "John", "Jennifer",
        "Michael", "Linda", "William", "Elizabeth", "David", "Barbara",
        "Richard", "Susan", "Joseph", "Jessica", "Thomas", "Sarah",
        "Charles", "Karen", "Christopher", "Nancy", "Daniel", "Lisa",
        "Matthew", "Betty", "Anthony", "Margaret", "Mark", "Sandra",
        "Donald", "Ashley", "Steven", "Kimberly", "Paul", "Emily",
        "Andrew", "Donna", "Joshua", "Michelle", "Kenneth", "Dorothy",
        "Kevin", "Carol", "Brian", "Amanda", "George", "Melissa",
        "Timothy", "Deborah", "Ronald", "Stephanie", "Edward", "Rebecca"
    }

    -- seed with current time once per run for randomness

    return names[math.random(1, #names)]
end

function pick(list)
    if #list == 0 then
        return nil
    end
    return list[math.random(1, #list)]
end

function random_int(max) 
    return math.random(1, max)
end
