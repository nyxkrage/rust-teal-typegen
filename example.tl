-- an enum: a set options of strings
local enum State
   "open"
   "closed"
end

local enum Empty
end

-- a record: a table with a known set of fields
local record Typings
    simple: number
    union: string | {string:number}
    arr: {number}
    arr2: {{string}}
    tuple: {number, string}
    map: {string:number}
    fn: function(number, number): number
    fn: function(number, number)
    fn: function()
end

-- generics: allows there to be used any type in place of T
local record Generic<T>
   x: T
   y: T
end

-- a record: a table with a known set of fields
local record Optionals
   x: number | nil
   y: number | integer | nil
end

-- an arrayrecord: a record which doubles as a record and an array
local record ArrayRecord
   {number}
   item: string
end
