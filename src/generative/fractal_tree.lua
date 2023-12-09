this.outlets = 3
--
local gl = require("opengl")
local GL = gl
--
MAX_LIMBS = 100
NPARAMS = 5
--
patcherID = nil
matParams = {}
matBranches = {}
matSway = {}
--
tree = {}
--
deviceActive = 1
releaseKill = false
nleaves = 20
limbThresh = 2
maxBranches = 60
branchLen = 0.032
minDist = 0.03
maxDist = 1
leafArea = { 0, 0, 1 }
rootArea = { 0, 0, 0.3 }
displayLeafArea = false
displayRootArea = false
growNextFrame = false
--
out_mat = 0
out_limbs = 1
out_dict = 2
--
--col_old = {1., 0.709804, 0.196078} --YELLOW
--col_new = {0.427451, 0.843137, 1.} --BLUE
col_active = { 1., 0.709804, 0.196078 }
col_inactive = { 0.314, 0.314, 0.314 }




-------------------------- FROM OUTSIDE

--
function activeCol(r, g, b)
    col_active = { r, g, b }
end

--
function inactiveCol(r, g, b)
    col_inactive = { r, g, b }
end

--
function set_nleaves(v)
    nleaves = v
end

--
function activeState(v)
    deviceActive = v
end

--
function set_maxDist(v)
    maxDist = v
end

--
function set_minLen(v)
    limbThresh = v
end

--
function set_maxBranch(v)
    maxBranches = v
end

--
function rootRange(x, y, r)
    rootArea = { x, y, r }
end

--
function leafRange(x, y, r)
    leafArea = { x, y, r }
end

--
function trigTree()
    newGrowth(tree)
end

--
function relKill(v)
    if v == 1 then
        releaseKill = true
    else
        releaseKill = false
    end
end

--
function saveTree(tree)
    if tree.exists then
        outlet(out_dict, "clearDict", "clear")
        tree:output(out_dict)
        --tree.trunk:printLimbs(tree)
    end
end

--
function prepareTreeLoad()
    if tree.exists then
    end
end

--
function loadLimb(limbIndex)
    tree:loadLimb(limbIndex, matBranches)
end

--
function updateAllParams()
    if tree.exists then
        tree:updateParams(matParams, self)
        outlet(out_mat, "bang")
        outlet(out_limbs, tree.totalLimbs)
        --tree.trunk:printLimbs(tree)
    end
end

-------------------------- MAIN FUNCTIONS

--
function setup(id)
    --
    math.randomseed(os.time())
    --
    patcherID = tostring(id)
    matParams = jit.matrix(patcherID .. "params")
    matBranches = jit.matrix(patcherID .. "branches")
    matSway = jit.matrix(patcherID .. "sway")
    --
    tree = Tree:new(branchLen)
    --
    --newGrowth(tree)
end

--
function newGrowth(tree)
    -- Get random position within area
    startPos = vector.new(rootArea[1], rootArea[2])
    startOffset = vector.rand()
    startOffset:multScalar(rootArea[3])
    startPos:add(startOffset)
    startPos:clamp(-1, 1)
    dir = vector.__sub(vector.new(0, 0), startPos)
    --
    tree:setup(nleaves, leafArea, startPos, dir, limbThresh)
end

--
function trigDeath(tree)
    if tree.exists then
        tree.decay = true
        tree.matured = true
    end
end

-------------------------- RENDER/FRAME

-- Run
function int()
    -- Tree Decay
    if tree.decay then
        tree:die()
        tree:updateParams(matParams)
        outlet(out_mat, "bang")
        outlet(out_limbs, tree.totalLimbs)
        -- Check if finished growing
    elseif not tree.matured then
        tree:grow(minDist, maxDist, maxBranches)
        tree:updateParams(matParams)
        outlet(out_mat, "bang")
        outlet(out_limbs, tree.totalLimbs)
        --
        if tree.matured then
            saveTree(tree)
        end
    end
end

--
function draw()
    tree:show(matParams, os.clock() % 1)
end

--
function leafAreaShow()
    x = leafArea[1]
    y = leafArea[2]
    r = leafArea[3]
    --
    gl.Color(0, 0.7, 0)
    gl.Begin(GL.QUADS)
    gl.Vertex(x - r, y - r) -- TL
    gl.Vertex(x + r, y - r) -- TR
    gl.Vertex(x + r, y + r) -- BR
    gl.Vertex(x - r, y + r) -- BL
    gl.End()
end

--
function rootAreaShow()
    x = rootArea[1]
    y = rootArea[2]
    r = rootArea[3]
    --
    gl.Color(0.7, 0.5, 0)
    gl.Begin(GL.QUADS)
    gl.Vertex(x - r, y - r) -- TL
    gl.Vertex(x + r, y - r) -- TR
    gl.Vertex(x + r, y + r) -- BR
    gl.Vertex(x - r, y + r) -- BL
    gl.End()
end

-------------------------- CLASSES / UTILITIES

--Tree

Tree = {
    trunk,
    leaves,
    len,
    trunkComplete,
    currentTrunk,
    matured,
    totalBranches,
    totalLimbs,
    limbThresh,
    decay,
    exists,
}
Tree.__index = Tree

--
function Tree:new(len)
    o = {}
    setmetatable(o, self)
    o.len = len
    o.matured = true
    o.decay = false
    o.exists = false
    return o
end

function Tree:setup(nleaves, leafArea, rootStart, rootDir, limbThresh)
    self.limbThresh = limbThresh
    self.totalLimbs = 0
    self.totalBranches = 0
    self.leaves = {}
    self.trunkComplete = false
    self.matured = false
    self.decay = false
    self.exists = true
    -- INIT
    for i = 1, nleaves do
        self.leaves[i] = Leaf:new(leafArea)
    end
    -- Add root branch to main limb
    root = Branch:new(rootStart, rootDir, self.len, nil, self.totalBranches)
    self.totalBranches = self.totalBranches + 1
    self.trunk = Limb:new(root, self.totalLimbs, nil, nil)
    self.totalLimbs = self.totalLimbs + 1
    self.currentTrunk = root
end

--
function Tree:growTrunk()
    newBranch = Branch:new(nil, nil, self.len, self.currentTrunk, self.totalBranches)
    self.totalBranches = self.totalBranches + 1
    table.insert(self.trunk.branches, newBranch)
    self.currentTrunk = newBranch
end

--
function Tree:closeEnough(b, minDist, maxDist)
    for l = 1, #self.leaves do
        dist = vector.distance(b.pos, self.leaves[l].pos)
        if dist < maxDist then
            return true
        end
    end
    return false
end

--
function Tree:grow(minDist, maxDist, maxBranches)
    -- Grow Trunk
    if not self.trunkComplete then
        self.trunkComplete = self:closeEnough(self.currentTrunk, minDist, maxDist)
        if not self.trunkComplete then
            self:growTrunk()
            return
        end
    end
    -- Attract/Grow Limbs
    isGrowing = false
    local nleaves = #self.leaves
    for l = 1, nleaves do
        -- For finding the closest branch to current leaf
        closest = { branch = nil, dir = nil, dist = -1 }
        -- Find closest branch
        self.trunk:attractBranch(self.leaves[l], closest, minDist, maxDist)
        -- Add vector towards the leaf to the closest branch from it
        if closest.branch ~= nil then
            closest.dir:normalize()
            closest.branch.dir:add(closest.dir)
            closest.branch.count = closest.branch.count + 1
            if not isGrowing then isGrowing = true end
        end
    end
    -- Remove leaves that have been reached
    for l = nleaves, 1, -1 do
        if self.leaves[l].reached then
            table.remove(self.leaves, l)
        end
    end
    --
    self.trunk:grow(self)
    -- Grow limbs
    self.matured = not isGrowing or self.totalLimbs >= maxBranches
end

--
function Tree:die()
    self.trunk:die(self)
end

--
function Tree:updateParams(matParams)
    if self.trunk then
        self.trunk:updateParams(matParams, self)
    end
end

--
function Tree:reorderLimbs(index)
    self.trunk:reorderLimbs(index)
end

--
function Tree:show(matParams, phase)
    if self.exists then
        if deviceActive == 0 then
            gl.Color(col_inactive)
        else
            gl.Color(col_active)
        end
        -- Draw Limbs
        self.trunk:show(matParams, self, 0, phase)
        --[[
    -- Draw Leaves
    gl.Color(0, 1, 0, 1)
    gl.Begin(GL.QUADS)
      for i = 1, #self.leaves do
        self.leaves[i]:show()
      end
    gl.End()
    --]]
    end
end

--
function Tree:output(outIndex)
    self.trunk:output(self, outIndex, "1")
end

--
function Tree:loadLimb(limbIndex, matBranches)
    --
    nentries = matBranches.dim
    vals = {}
    for i = 1, nentries do
        table.insert(vals, getMatrixCellSingle(matBranches:getcell(i - 1)))
    end
    -- index 1 is the root
    if string.find(limbIndex, "_") == nil then
        self.totalLimbs = 0
        self.exists = true
        self.matured = true
        self.decay = false
        self.trunk = {}
        --
        self.totalBranches = vals[1] --ParentIndex is #totalBranches for root
        table.remove(vals, 1)
        paramIndex = vals[1]
        table.remove(vals, 1)
        --
        newPos = vector.new(vals[1], vals[2])
        newDir = vector.new(vals[3], vals[4])
        branchGen = vals[5]
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        baseBranch = Branch:new(newPos, newDir, self.len, nil, branchGen)
        self.trunk = Limb:new(baseBranch, paramIndex, nil, nil)
        self.totalLimbs = self.totalLimbs + 1
        self.trunk:loadBranches(self, vals)
    else
        limbIndex = string.sub(limbIndex, 3)
        self.trunk:loadLimb(self, limbIndex, vals)
    end
end

-- Limb
Limb = {
    paramIndex,
    limbs,
    branches,
    len,
    w,
    x,
    y,
    freq,
    decay,
    amp,
    detune,
    sway,
    isGrowing,
    parent,
    baseBranchIndex
}
Limb.__index = Limb
--
function Limb:new(baseBranch, index, parent, baseBranchIndex)
    o = {}
    setmetatable(o, self)
    o.paramIndex = index
    o.parent = parent
    o.limbs = {}
    o.branches = { baseBranch }
    o.baseBranchIndex = baseBranchIndex
    o.isGrowing = true
    o.w = baseBranch.gen
    o.sway = 0
    return o
end

--
function Limb:attractBranch(leaf, closest, minDist, maxDist)
    -- Attract to own limbs
    for li = 1, #self.limbs do
        if self.limbs[li]:attractBranch(leaf, closest, minDist, maxDist) then
            return true
        end
    end
    -- Attract to own branches
    for b = 1, #self.branches do
        -- Distance from leaf
        dir = vector.__sub(leaf.pos, self.branches[b].pos)
        dist = dir:mag()
        -- leaf has been reached (set it to be removed)
        if dist < minDist then
            leaf.reached = true
            closest.branch = nil
            return true
            -- out of range
        elseif dist > maxDist then
            -- Closest branch to leaf (save direction vector)
        elseif closest.branch == nil or dist < closest.dist then
            closest.branch = self.branches[b]
            closest.dir = dir
            closest.dist = dist
        end
    end
    --
    return false
end

--
function Limb:grow(tree)
    -- Grow own limbs
    for li = #self.limbs, 1, -1 do
        self.limbs[li]:grow(tree)
    end
    --  Grow branches
    isGrowing = false
    for b = #self.branches, 1, -1 do
        branch = self.branches[b]
        -- If one or more leaves attracting the branch, add a new branch
        if branch.count > 0 then
            -- Check if limb is still growing
            if not isGrowing then isGrowing = true end
            -- Scale vector attractors
            branch.dir:divScalar(branch.count)
            -- Workaround for opposite direction vectors
            rand = vector.rand()
            rand:multScalar(0.25)
            branch.dir:add(rand)
            -- Normalize direction vector
            branch.dir:normalize()
            -- Add new branch or limb
            baseBranch = Branch:new(nil, nil, tree.len, branch, tree.totalBranches)
            tree.totalBranches = tree.totalBranches + 1
            if branch.branched then -- Add new limb if branch splits
                table.insert(self.limbs, Limb:new(baseBranch, tree.totalLimbs, self, b))
                tree.totalLimbs = tree.totalLimbs + 1
            else -- Add new Branch (continuing limb)
                table.insert(self.branches, baseBranch)
                branch.branched = true
            end
            branch:reset()
        end
    end
    --
    self:stoppedGrowing(tree)
end

--
function Limb:stoppedGrowing(tree)
    if self.isGrowing and not isGrowing then
        -- Remove if stopped growing below min length
        limbLen = self.len
        if limbLen == nil or (limbLen < tree.limbThresh and #self.limbs == 0) then
            -- reorder any new limbs that appeared after this
            tree:reorderLimbs(self.paramIndex)
            -- Remove small limb
            table.remove(self.parent.limbs, #self.parent.limbs)
            tree.totalLimbs = tree.totalLimbs - 1
        end
        self.isGrowing = false
    end
end

--
function Limb:die(tree)
    -- Kill own limbs
    for li = #self.limbs, 1, -1 do
        self.limbs[li]:die(tree)
    end
    -- Remove branches
    table.remove(self.branches, #self.branches)
    tree.totalBranches = tree.totalBranches - 1
    -- If no more branches, remove limb
    if #self.branches == 0 and #self.limbs == 0 then
        -- If this was the trunk of tree then destroy tree
        if self.parent == nil then
            tree.trunk = nil
            tree.totalLimbs = 0
            tree.decay = false
            tree.exists = false
        else
            table.remove(self.parent.limbs, #self.parent.limbs)
            tree.totalLimbs = tree.totalLimbs - 1
        end
    end
end

--
function Limb:reorderLimbs(index)
    if self.paramIndex > index then
        self.paramIndex = self.paramIndex - 1
    end
    --
    for li = 1, #self.limbs do
        self.limbs[li]:reorderLimbs(index)
    end
end

--
function Limb:updateParams(matParams, tree)
    nbranches = #self.branches
    baseBranch = self.branches[1]
    tipBranch = self.branches[nbranches]
    --
    if not baseBranch then return end
    --
    self.len = nbranches
    self.x = (baseBranch.pos.x + tipBranch.pos.x) * 0.5
    self.y = (baseBranch.pos.y + tipBranch.pos.y) * 0.5
    -- Convert attributes to DSP parameters
    self:convert(tree)
    -- Add params to matrix for DSP use
    matParams:setcell1d(
        self.paramIndex, self.freq, self.decay, self.amp,
        self.x, self.y, self.detune, 0
    )
    -- Update own limbs
    for li = 1, #self.limbs do
        self.limbs[li]:updateParams(matParams, tree)
    end
end

--
function Limb:convert(tree)
    len2w = 2 / tree.len
    maxArea = 240
    --
    r = ((tree.totalBranches - self.w) / tree.totalBranches) * math.pow(tree.totalBranches / len2w, 0.2)
    a = (math.pi * r * (r + math.sqrt(self.len * self.len + r * r))) / maxArea
    --
    self.freq = (1 - a) * 2 - 1
    self.decay = math.pow(self.len, 1.5) / (r * 3) * 100
    self.amp = math.pow(a, 1.5) * 1.23 + 0.03
    --self.detune = 0
end

--
function Limb:show(matParams, tree, order, phase)
    --
    if not self.branches[1] or not self.freq then return end
    --
    orderCoef = (order / 5) * 0.7
    -- Parent branch movement offset
    moveOff = {}
    if self.branches[1].parent then
        connBranch = self.branches[1].parent
        moveOff = vector.__sub(connBranch.movePos, connBranch.pos)
    else
        moveOff = vector.new(0, 0)
    end
    -- Sine Movement
    params = getMatrixCell(matParams:getcell(self.paramIndex))
    sway = math.min(matSway:getcell(self.paramIndex) * 2, 0.1)
    move = 0
    dir = vector.new(0, 0)
    if sway > 0 then
        move = sineLookup((phase * (self.freq + 1) * 4) % 1) * sway * 2
        -- Movement Dir
        dir = self.branches[1].dir:clone()
        dir:rotate(math.rad(90))
        dir:normalize()
    end
    -- Color Alpha
    --alpha = (10*sway*0.35 + 0.65)
    -- Render Branches
    nbranches = #self.branches
    for b = 1, nbranches do
        branch = self.branches[b]
        if branch.parent ~= nil then
            -- Coefs
            ac = branch.gen / tree.totalBranches
            bc = orderCoef + ac
            cc = (b - 1) / (nbranches - 1)
            -- Move pos with sway
            branch.movePos = branch.pos:clone()
            v1 = branch.movePos
            v2 = branch.parent.movePos:clone()
            v1:add(moveOff)
            v1:add(vector.__mul(dir, cc * move))
            -- Stylize and draw
            gl.LineWidth(math.max((1 - ac) * math.pow(tree.totalBranches * tree.len * 1.4, 0.6), 1))
            --[[
      if deviceActive == 0 then
        gl.Color(col_inactive)
      else
        bcmin = 1-bc
        gl.Color(
          alpha * (col_old[1]*bcmin + col_new[1]*bc),
          alpha * (col_old[2]*bcmin + col_new[2]*bc),
          alpha * (col_old[3]*bcmin + col_new[3]*bc)
        )
      end
      ]]
            gl.Begin(GL.LINES)
            gl.Vertex(v1.x, v1.y)
            gl.Vertex(v2.x, v2.y)
            gl.End()
        end
    end
    -- Show own limbs
    for li = 1, #self.limbs do
        self.limbs[li]:show(matParams, tree, order + 1, phase)
    end
end

--
function Limb:printLimbs(tree)
    print(
        "TotalL", tree.totalLimbs,
        "totalB", tree.totalBranches, "id", self.paramIndex,
        "len", self.len, "w", self.w,
        "freq", self.freq, "decay", self.decay,
        "amp", self.amp
    )
    --
    for li = 1, #self.limbs do
        self.limbs[li]:printLimbs(tree)
    end
end

-- Save
function Limb:output(tree, outIndex, limbIndex)
    nbranches = #self.branches
    nbranches5 = nbranches * 5
    matBranches.dim = nbranches5
    matBranches:clear()
    -- Output Branches
    for b = 1, nbranches do
        b5 = (b - 1) * 5
        pos = self.branches[b].pos:unpack()
        dir = self.branches[b].dir:unpack()
        matBranches:setcell1d(b5, pos[1])
        matBranches:setcell1d(b5 + 1, pos[2])
        matBranches:setcell1d(b5 + 2, dir[1])
        matBranches:setcell1d(b5 + 3, dir[2])
        matBranches:setcell1d(b5 + 4, self.branches[b].gen)
    end
    -- Since the trunk has no parent index, store the total branch # here
    parentIndex = self.baseBranchIndex
    if parentIndex == nil then
        parentIndex = tree.totalBranches
    end
    --
    outlet(
        outIndex, "listlength", nbranches5,
        "set", "replace", limbIndex,
        parentIndex, self.paramIndex
    )
    -- Output limbs
    limbIndex = limbIndex .. "_"
    for li = 1, #self.limbs do
        self.limbs[li]:output(tree, outIndex, limbIndex .. tostring(li))
    end
end

--
function Limb:loadLimb(tree, limbIndex, vals)
    --
    if string.find(limbIndex, "_") == nil then
        parentIndex = vals[1]
        table.remove(vals, 1)
        paramIndex = vals[1]
        table.remove(vals, 1)
        parent = self.branches[parentIndex]
        --
        newPos = vector.new(vals[1], vals[2])
        newDir = vector.new(vals[3], vals[4])
        branchGen = vals[5]
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        table.remove(vals, 1)
        baseBranch = Branch:new(newPos, newDir, tree.len, parent, branchGen)
        self.limbs[tonumber(limbIndex)] = Limb:new(baseBranch, paramIndex, self, parentIndex)
        tree.totalLimbs = tree.totalLimbs + 1
        --
        self.limbs[#self.limbs]:loadBranches(tree, vals)
    else
        parentIndex = tonumber(string.sub(limbIndex, 1, 1))
        limbIndex = string.sub(limbIndex, 3)
        self.limbs[parentIndex]:loadLimb(tree, limbIndex, vals)
    end
end

--
function Limb:loadBranches(tree, branchPos)
    for b = 0, #branchPos / 5 - 1 do
        b5 = b * 5
        lastBranch = self.branches[#self.branches]
        newPos = vector.new(branchPos[b5 + 1], branchPos[b5 + 2])
        newDir = vector.new(branchPos[b5 + 3], branchPos[b5 + 4])
        branchGen = branchPos[b5 + 5]
        newBranch = Branch:new(newPos, newDir, tree.len, lastBranch, branchGen)
        table.insert(self.branches, newBranch)
    end
end

-- Branch
Branch = {
    parent,
    pos,
    dir,
    count,
    saveDir,
    len,
    branched,
    gen,
    movePos,
}
Branch.__index = Branch
--
function Branch:new(pos, dir, len, parent, gen)
    o = {}
    setmetatable(o, self)
    o.count = 0
    o.len = len
    o.branched = false
    o.gen = gen
    --
    if parent then
        o.parent = parent
        --
        if pos then
            o.pos = pos
        else
            o.pos = o.parent:next()
        end
        --
        if dir then
            o.dir = dir
        else
            o.dir = o.parent.dir:clone()
        end
    else
        o.parent = nil
        o.pos = pos:clone()
        if dir then
            o.dir = dir:clone()
        else
            o.dir = vector.new(0, 0)
        end
    end
    --
    o.movePos = o.pos:clone()
    o.saveDir = o.dir:clone()
    return o
end

--
function Branch:reset()
    self.count = 0
    self.dir = self.saveDir:clone()
end

--
function Branch:next()
    v = vector.__mul(self.dir, self.len)
    v = vector.__add(self.pos, v)
    return v
end

-- Leaf
Leaf = { pos, reached }
Leaf.__index = Leaf
--
function Leaf:new(area)
    o = {}
    setmetatable(o, self)
    o.reached = false
    --
    o.pos = vector.new(area[1], area[2])
    startOffset = vector.rand()
    startOffset:multScalar(area[3])
    o.pos:add(startOffset)
    o.pos:clamp(-1, 1)
    return o
end

--
function Leaf:show()
    gl.Vertex(self.pos.x - 0.01, self.pos.y - 0.01) -- TL
    gl.Vertex(self.pos.x + 0.01, self.pos.y - 0.01) -- TR
    gl.Vertex(self.pos.x + 0.01, self.pos.y + 0.01) -- BR
    gl.Vertex(self.pos.x - 0.01, self.pos.y + 0.01) -- BL
end

-- Vector
vector = {}
vector.__index = vector
--
local function is_vector(t)
    return getmetatable(t) == vector
end
--
function vector.new(x, y)
    return setmetatable({ x = x or 0, y = y or 0 }, vector)
end

-- operator overloading
function vector.__add(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected.")
    return vector.new(lhs.x + rhs.x, lhs.y + rhs.y)
end

--
function vector.__sub(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected.")
    return vector.new(lhs.x - rhs.x, lhs.y - rhs.y)
end

--
function vector.__mul(lhs, rhs)
    local is_rhs_vector = is_vector(rhs)
    local is_lhs_vector = is_vector(lhs)
    if type(lhs) == "number" and is_rhs_vector then
        return vector.new(rhs.x * lhs, rhs.y * lhs)
    elseif type(rhs) == "number" and is_lhs_vector then
        return vector.new(lhs.x * rhs, lhs.y * rhs)
    elseif is_rhs_vector and is_lhs_vector then
        return vector.new(lhs.x * rhs.x, lhs.y * rhs.y)
    else
        error("Type mismatch: vector and/or number expected", 2)
    end
end

--
function vector.__unm(t)
    assert(is_vector(t), "Type mismatch: vector expected.")
    return vector.new(-t.x, -t.y)
end

--
function vector:__tostring()
    return "(" .. self.x .. ", " .. self.y .. ")"
end

--
function vector.__eq(lhs, rhs)
    return lhs.x == rhs.x and lhs.y == rhs.y
end

--
function vector.__lt(lhs, rhs)
    return lhs.x < rhs.x or (not (rhs.x < lhs.x) and lhs.y < rhs.y)
end

--
function vector.__le(lhs, rhs)
    return lhs.x <= rhs.x or lhs.y <= rhs.y
end

-- actual functions
function vector:clone()
    return vector.new(self.x, self.y)
end

--
function vector:length()
    return math.sqrt(self.x * self.x + self.y * self.y)
end

--
function vector:length_squared()
    return self.x * self.x + self.y * self.y
end

--
function vector:is_unit()
    return self:length_squared() == 1
end

--
function vector:unpack()
    return { self.x, self.y }
end

--
function vector:normalize()
    local len = self:length()
    if len ~= 0 and len ~= 1 then
        self.x = self.x / len
        self.y = self.y / len
    end
end

--
function vector:normalized()
    return self:clone():normalize()
end

--
function vector:multScalar(v)
    self.x = self.x * v
    self.y = self.y * v
end

--
function vector:divScalar(v)
    self.x = self.x / v
    self.y = self.y / v
end

--
function vector:addScalar(v)
    self.x = self.x + v
    self.y = self.y + v
end

--
function vector:add(v)
    self.x = self.x + v.x
    self.y = self.y + v.y
end

--
function vector:mag()
    return math.sqrt(self.x * self.x + self.y * self.y)
end

--
function vector:rotate(rad)
    px = self.x
    self.x = px * math.cos(rad) - self.y * math.sin(rad)
    self.y = px * math.sin(rad) + self.y * math.cos(rad)
end

--
function vector:clamp(lo, hi)
    self.x = math.min(math.max(self.x, lo), hi)
    self.y = math.min(math.max(self.y, lo), hi)
end

--
function vector.dot(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected")
    return lhs.x * rhs.x + lhs.y * rhs.y
end

--
function vector.distance(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected")
    local dx, dy = lhs.x - rhs.x, lhs.y - rhs.y
    return math.sqrt(dx * dx + dy * dy)
end

--
function vector.distance_squared(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected")
    local dx, dy = lhs.x - rhs.x, lhs.y - rhs.y
    return dx * dx + dy * dy
end

--
function vector.max(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected")
    local x = math.max(lhs.x, rhs.x)
    local y = math.max(lhs.y, rhs.y)
    return vector.new(x, y)
end

--
function vector.min(lhs, rhs)
    assert(is_vector(lhs) and is_vector(rhs), "Type mismatch: vector expected")
    local x = math.min(lhs.x, rhs.x)
    local y = math.min(lhs.y, rhs.y)
    return vector.new(x, y)
end

--
function vector.angle(from, to)
    assert(is_vector(from) and is_vector(to), "Type mismatch: vector expected")
    return math.acos(vector.dot(from, to) / (from:length() * to:length()))
end

--
function vector.direction(from, to)
    assert(is_vector(from) and is_vector(to), "Type mismatch: vector expected")
    return math.atan2(to.y - from.y, to.x - from.y)
end

--
function vector.lerp(from, to, t)
    assert(is_vector(from) and is_vector(to), "Type mismatch: vector expected")
    assert(type(t) == "number", "Type mismatch: number expected for t")
    return from * t + (to * (1 - t))
end

--
function vector.rand()
    local x = math.random() * 2 - 1
    local y = math.random() * 2 - 1
    return vector.new(x, y)
end

--
function getMatrixCell(...)
    return { ... }
end

--
function getMatrixCellSingle(v)
    return v
end

--
function sineLookup(phase)
    tableSize = 511
    phaseIndex = math.floor(phase * tableSize + 1)
    return sineTable[phaseIndex]
end

--
function getSineTable(tableSize)
    st = {}
    for i = 1, tableSize do
        phase = (i - 1) / (tableSize - 1)
        st[i] = math.sin(math.pi * 2 * phase) * 0.2
    end
    return st
end

--
sineTable = getSineTable(512)
