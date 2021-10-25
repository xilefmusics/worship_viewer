const SCALES = {
    'sharp': ['A', 'A#', 'B', 'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#'],
    'flat': ['A', 'Bb', 'Cb', 'C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab'],
};

const splitLine = line => {
    const isStartOfTransposable = c => c === 'A' || c === 'B' || c === 'H' || c === 'C' || c === 'D' || c === 'E' || c === 'F' || c === 'G';
    const makeStartWithTransposable = line => {
        let list = [];
        let current = '';
        for (const c of line) {
            if (isStartOfTransposable(c)) {
                list.push(current);
                current = '';
            }
            current += c;
        }
        list.push(current);
        if (list[0] === '') {
            list.shift();
        }
        return list;
    }
    const splitTransposable = item => {
        const findIdxToSplit = item => {
            if (item.length > 0 && isStartOfTransposable(item.charAt(0))) {
                if (item.length > 1 && (item.charAt(1) === '#' || item.charAt(1) === 'b')) {
                    return 2;
                }
                return 1;
            }
            return 0;
        }
        const idx = findIdxToSplit(item);
        return {
            'transposable': item.substring(0, idx),
            'notTransposable': item.substring(idx),
        }
    }
    return makeStartWithTransposable(line).map(item => splitTransposable(item));
}

const getLevel = transposable => {
    const first = transposable.charAt(0);
    let level = 0;
    if (first === 'A') {level = 0}
    else if (first === 'B') {level = 2}
    else if (first === 'C') {level = 3}
    else if (first === 'D') {level = 5}
    else if (first === 'E') {level = 7}
    else if (first === 'F') {level = 8}
    else if (first === 'G') {level = 10}
    if (transposable.length > 1 && transposable.charAt(1) === 'b') { level -= 1};
    if (transposable.length > 1 && transposable.charAt(1) === '#') { level += 1};
    return (level + 12) % 12;
};

const transposeSplit = (split, halftones, isSharp) => {
    const transposeItem = (item, halftones, isSharp) => {
        if (item.transposable === '') {
            return item;
        }
        const level = (getLevel(item.transposable)+halftones+12) % 12;
        let newTransposable = null;
        if (isSharp) {
            newTransposable = SCALES.sharp[level];
        } else {
            newTransposable = SCALES.flat[level];
        }
        return {
            'transposable': newTransposable,
            'notTransposable': item.notTransposable,
        };
    }
    return split.map(item => transposeItem(item, halftones, isSharp));
};

const joinLine = split => {
    let line = '';
    for (const item of split) {
        line += (item.transposable + item.notTransposable);
    }
    return line;
};

const getIsSharpKey = key => key === 'C' || key === 'D' || key === 'E' || key === 'G' || key === 'A' || key === 'B';
const getIsSharpLevel = level => level === 0 || level === 2 || level === 3 || level === 5 || level === 7 || level === 10;

const transposeLine = (line, oldKey, newKey) => {
    let halftones = null;
    let isSharp = null;
    if (newKey.startsWith('Self:')) {
        halftones = parseInt(newKey.substring(5));
        isSharp = getIsSharpLevel((getLevel(oldKey) + halftones + 12) % 12);
        
    } else {
        halftones = (getLevel(newKey)-getLevel(oldKey) + 12) % 12;
        isSharp = getIsSharpKey(newKey);
    }
    return joinLine(transposeSplit(splitLine(line), halftones, isSharp));
};

export default function(song, newKey) {
    console.log({
        'newKey': newKey,
        'song': song,
    })
    if (newKey === 'Self') {
        return song;
    }
    let newSong = {...song};
    for (let section of newSong.sections) {
        for (let line of section.lines) {
            line.chord = transposeLine(line.chord, song.key, newKey);
        }
    }
    return newSong;
};