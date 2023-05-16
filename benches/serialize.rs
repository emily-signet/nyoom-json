use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use nyoom_json::{JsonBuffer, UnescapedStr, WriteToJson};
use serde::{Deserialize, Serialize};

macro_rules! noescape {
    ($s:expr) => {
        UnescapedStr::create($s)
    };
}

macro_rules! ser_arr {
    ($into:expr, $from:expr) => {
        for val in $from {
            $into.add(val.as_str());
        }
    };
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    license: RootLicense,
    repository: String,
    last_update: String,
    data: Vec<Anime>,
}

#[inline(always)]
pub fn write_root<S: JsonBuffer>(root: &Root, out: &mut nyoom_json::Serializer<S>) {
    let mut obj = out.object();
    obj.complex_field(UnescapedStr::create("license"), |field| {
        let mut f = field.object();
        f.field(noescape!("name"), root.license.name.as_str());
        f.field(noescape!("url"), root.license.url.as_str());
    });

    obj.field(noescape!("repository"), root.repository.as_str());
    obj.field(noescape!("lastUpdate"), root.last_update.as_str());
    let mut anime_array = obj.array_field(noescape!("data"));
    for series in root.data.iter() {
        let mut anime = anime_array.add_object();
        anime.field(noescape!("title"), series.title.as_str());
        anime.field(noescape!("episodes"), series.episodes);
        anime.field(noescape!("thumbnail"), series.thumbnail.as_str());
        anime.field(noescape!("picture"), series.picture.as_str());
        anime.complex_field(noescape!("sources"), |ser| {
            let mut arr = ser.array();
            ser_arr!(arr, &series.sources);
        });
        anime.complex_field(noescape!("synonyms"), |ser| {
            let mut arr = ser.array();
            ser_arr!(arr, &series.synonyms);
        });
        anime.complex_field(noescape!("relations"), |ser| {
            let mut arr = ser.array();
            ser_arr!(arr, &series.relations);
        });
        anime.complex_field(noescape!("tags"), |ser| {
            let mut arr = ser.array();
            ser_arr!(arr, &series.tags);
        });
        anime.complex_field(noescape!("animeSeason"), |ser| {
            let mut obj = ser.object();
            obj.field(noescape!("season"), series.anime_season.season);
            obj.field(noescape!("year"), series.anime_season.year);
        });
        anime.field(noescape!("type"), series.ty);
        anime.field(noescape!("status"), series.status);
        anime.end();
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootLicense {
    name: String,
    url: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    sources: Vec<String>,
    title: String,
    #[serde(rename = "type")]
    ty: AnimeType,
    episodes: i64,
    status: AnimeStatus,
    anime_season: AnimeSeason,
    picture: String,
    thumbnail: String,
    synonyms: Vec<String>,
    relations: Vec<String>,
    tags: Vec<String>,
}

#[derive(Copy, Clone, Deserialize, Serialize, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeType {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
    Unknown,
}

impl<S: JsonBuffer> WriteToJson<S> for AnimeType {
    fn write_to_json(self, out: &mut S) {
        UnescapedStr::create(self.as_ref()).write_to_json(out)
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeStatus {
    Finished,
    Ongoing,
    Upcoming,
    Unknown,
}

impl<S: JsonBuffer> WriteToJson<S> for AnimeStatus {
    fn write_to_json(self, out: &mut S) {
        UnescapedStr::create(self.as_ref()).write_to_json(out)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeSeason {
    season: AnimeSeasons,
    year: Option<i64>,
}

#[derive(Copy, Clone, Deserialize, Serialize, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AnimeSeasons {
    Spring,
    Summer,
    Fall,
    Winter,
    Undefined,
}

impl<S: JsonBuffer> WriteToJson<S> for AnimeSeasons {
    fn write_to_json(self, out: &mut S) {
        UnescapedStr::create(self.as_ref()).write_to_json(out)
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut anime: Root = serde_json::from_str(include_str!("anime-test.json")).unwrap();
    anime.data.truncate(200);
    let mut group = c.benchmark_group("serialization");

    let size = serde_json::to_string(&anime).unwrap().len();

    group.measurement_time(Duration::from_secs(20));
    group.throughput(criterion::Throughput::BytesDecimal(size as u64));
    group.bench_function("serde", |b| {
        b.iter_with_large_drop(|| {
            let mut out = Vec::with_capacity(size);
            serde_json::to_writer(&mut out, &anime);
            out
        })
    });
    group.bench_function("nyoom", |b| {
        b.iter_with_large_drop(|| {
            let mut out = String::with_capacity(size);
            let mut ser = nyoom_json::Serializer::new(&mut out);
            write_root(&anime, &mut ser);
            out
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
